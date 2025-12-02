//! Query building and faceted search for papers.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, RangeQuery};
use tantivy::schema::Field;
use tantivy::schema::Value;
use tantivy::{DateTime, Searcher, TantivyDocument};

use crate::search::index::SearchIndex;

/// Search query parameters
#[derive(Deserialize, Debug, Default)]
pub struct SearchParams {
    /// Full-text search query
    pub q: Option<String>,
    /// Pagination limit
    pub limit: Option<i64>,
    /// Pagination offset
    pub offset: Option<i64>,
    /// Order by field (relevance, published_date)
    pub order_by: Option<String>,
    /// Order direction (asc, desc)
    pub order: Option<String>,
    /// Filter: papers published on or after this date
    pub date_from: Option<NaiveDate>,
    /// Filter: papers published on or before this date
    pub date_to: Option<NaiveDate>,
    /// Legacy search param (maps to q)
    pub search: Option<String>,
}

impl SearchParams {
    /// Get the effective search query (q takes precedence over search)
    pub fn get_query(&self) -> Option<&str> {
        self.q.as_deref().or(self.search.as_deref())
    }
}

/// Date bucket for histogram facets
#[derive(Serialize, Debug, Clone)]
pub struct DateBucket {
    pub year: i32,
    pub month: u32,
    pub count: u64,
}

/// Faceted search results
#[derive(Serialize, Debug, Clone)]
pub struct SearchFacets {
    pub date_histogram: Vec<DateBucket>,
}

/// Search response with papers, total hits, and facets
#[derive(Serialize, Debug)]
pub struct SearchResponse<T> {
    pub papers: Vec<T>,
    pub total_hits: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<SearchFacets>,
}

/// Result of a Tantivy search containing paper IDs
pub struct TantivySearchResult {
    pub paper_ids: Vec<uuid::Uuid>,
    pub total_hits: usize,
    pub facets: Option<SearchFacets>,
}

/// Execute a search query against the Tantivy index.
pub fn search_papers(
    search_index: &SearchIndex,
    query_str: &str,
    params: &SearchParams,
    limit: usize,
    offset: usize,
) -> Result<TantivySearchResult> {
    let searcher = search_index.reader.searcher();
    let fields = &search_index.fields;

    // Build query parser for full-text search across title, abstract, authors
    let query_parser = QueryParser::for_index(
        &search_index.index,
        vec![fields.title, fields.abstract_field, fields.authors],
    );

    let text_query = query_parser
        .parse_query(query_str)
        .context("Failed to parse search query")?;

    // Apply date range filter if provided
    let final_query: Box<dyn Query> =
        if params.date_from.is_some() || params.date_to.is_some() {
            let range_query = build_date_range_query(
                fields.published_date,
                params.date_from,
                params.date_to,
            );
            Box::new(BooleanQuery::new(vec![
                (Occur::Must, text_query),
                (Occur::Must, range_query),
            ]))
        } else {
            text_query
        };

    // Execute search - fetch more than needed to get total count
    let top_docs = searcher
        .search(&final_query, &TopDocs::with_limit(offset + limit + 1000))
        .context("Search failed")?;

    let total_hits = top_docs.len();

    // Extract paper IDs from results
    let paper_ids: Vec<uuid::Uuid> = top_docs
        .iter()
        .skip(offset)
        .take(limit)
        .filter_map(|(_, doc_address)| {
            let doc: TantivyDocument = searcher.doc(*doc_address).ok()?;
            let id_str = doc.get_first(fields.id)?.as_str()?;
            uuid::Uuid::parse_str(id_str).ok()
        })
        .collect();

    // Collect date facets
    let facets = collect_date_facets(&searcher, &top_docs, fields.published_date)?;

    Ok(TantivySearchResult {
        paper_ids,
        total_hits,
        facets: Some(facets),
    })
}

/// Build a date range query for filtering.
fn build_date_range_query(
    _date_field: Field,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Box<dyn Query> {
    let from_dt = from.map(|d| {
        DateTime::from_timestamp_secs(
            d.and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0),
        )
    });

    let to_dt = to.map(|d| {
        DateTime::from_timestamp_secs(
            d.and_hms_opt(23, 59, 59)
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0),
        )
    });

    Box::new(RangeQuery::new_date_bounds(
        "published_date".to_string(),
        from_dt.map(std::ops::Bound::Included).unwrap_or(std::ops::Bound::Unbounded),
        to_dt.map(std::ops::Bound::Included).unwrap_or(std::ops::Bound::Unbounded),
    ))
}

/// Collect date histogram facets from search results.
fn collect_date_facets(
    searcher: &Searcher,
    top_docs: &[(f32, tantivy::DocAddress)],
    date_field: Field,
) -> Result<SearchFacets> {
    let mut date_counts: HashMap<(i32, u32), u64> = HashMap::new();

    // Sample up to 10k docs for facets
    for (_, doc_address) in top_docs.iter().take(10000) {
        if let Ok(doc) = searcher.doc::<TantivyDocument>(*doc_address) {
            if let Some(date_val) = doc.get_first(date_field) {
                if let Some(dt) = date_val.as_datetime() {
                    let timestamp = dt.into_timestamp_secs();
                    if let Some(naive_dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                        let year = naive_dt.format("%Y").to_string().parse::<i32>().unwrap_or(0);
                        let month = naive_dt.format("%m").to_string().parse::<u32>().unwrap_or(0);
                        *date_counts.entry((year, month)).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    let mut date_histogram: Vec<DateBucket> = date_counts
        .into_iter()
        .map(|((year, month), count)| DateBucket { year, month, count })
        .collect();

    // Sort by date descending
    date_histogram.sort_by(|a, b| (b.year, b.month).cmp(&(a.year, a.month)));

    Ok(SearchFacets { date_histogram })
}
