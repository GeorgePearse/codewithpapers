//! Tantivy index management and document conversion.

use anyhow::{Context, Result};
use std::path::Path;
use tantivy::schema::Schema;
use tantivy::{Index, IndexReader, IndexWriter, TantivyDocument};

use crate::search::schema::{create_paper_schema, PaperFields};
use crate::Paper;

/// Wrapper around Tantivy index with schema and reader.
pub struct SearchIndex {
    pub index: Index,
    pub reader: IndexReader,
    pub schema: Schema,
    pub fields: PaperFields,
}

impl SearchIndex {
    /// Open an existing index from disk.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (schema, fields) = create_paper_schema();

        let index = Index::open_in_dir(path.as_ref())
            .with_context(|| format!("Failed to open index at {:?}", path.as_ref()))?;

        // Register the English stemming tokenizer
        let tokenizer_manager = index.tokenizers();
        tokenizer_manager.register(
            "en_stem",
            tantivy::tokenizer::TextAnalyzer::builder(tantivy::tokenizer::SimpleTokenizer::default())
                .filter(tantivy::tokenizer::RemoveLongFilter::limit(40))
                .filter(tantivy::tokenizer::LowerCaser)
                .filter(tantivy::tokenizer::Stemmer::new(tantivy::tokenizer::Language::English))
                .build(),
        );

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        Ok(Self {
            index,
            reader,
            schema,
            fields,
        })
    }

    /// Create a new index at the given path.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (schema, fields) = create_paper_schema();

        std::fs::create_dir_all(path.as_ref())
            .with_context(|| format!("Failed to create index directory at {:?}", path.as_ref()))?;

        let index = Index::create_in_dir(path.as_ref(), schema.clone())
            .with_context(|| format!("Failed to create index at {:?}", path.as_ref()))?;

        // Register the English stemming tokenizer
        let tokenizer_manager = index.tokenizers();
        tokenizer_manager.register(
            "en_stem",
            tantivy::tokenizer::TextAnalyzer::builder(tantivy::tokenizer::SimpleTokenizer::default())
                .filter(tantivy::tokenizer::RemoveLongFilter::limit(40))
                .filter(tantivy::tokenizer::LowerCaser)
                .filter(tantivy::tokenizer::Stemmer::new(tantivy::tokenizer::Language::English))
                .build(),
        );

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        Ok(Self {
            index,
            reader,
            schema,
            fields,
        })
    }

    /// Open existing index or create if it doesn't exist.
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().join("meta.json").exists() {
            Self::open(path)
        } else {
            Self::create(path)
        }
    }

    /// Create an IndexWriter with the given heap size (in bytes).
    pub fn writer(&self, heap_size: usize) -> Result<IndexWriter> {
        self.index
            .writer(heap_size)
            .context("Failed to create index writer")
    }

    /// Convert a Paper to a Tantivy document.
    pub fn paper_to_document(&self, paper: &Paper) -> TantivyDocument {
        let mut doc = TantivyDocument::new();

        // ID (stored for lookup)
        doc.add_text(self.fields.id, &paper.id.to_string());

        // Full-text fields
        doc.add_text(self.fields.title, &paper.title);

        if let Some(ref abstract_text) = paper.r#abstract {
            doc.add_text(self.fields.abstract_field, abstract_text);
        }

        // Flatten authors JSON array to searchable text
        if let Some(ref authors) = paper.authors {
            if let Some(arr) = authors.as_array() {
                let authors_text: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                doc.add_text(self.fields.authors, &authors_text.join(" "));
            }
        }

        // Exact match fields
        if let Some(ref arxiv_id) = paper.arxiv_id {
            doc.add_text(self.fields.arxiv_id, arxiv_id);
        }

        // Date field
        if let Some(date) = paper.published_date {
            let datetime = tantivy::DateTime::from_timestamp_secs(
                date.and_hms_opt(0, 0, 0)
                    .map(|dt| dt.and_utc().timestamp())
                    .unwrap_or(0),
            );
            doc.add_date(self.fields.published_date, datetime);
        }

        doc
    }
}

impl Clone for SearchIndex {
    fn clone(&self) -> Self {
        // Schema and fields are cheap to clone
        // Reader can be cloned (it's reference counted internally)
        Self {
            index: self.index.clone(),
            reader: self.reader.clone(),
            schema: self.schema.clone(),
            fields: PaperFields {
                id: self.fields.id,
                title: self.fields.title,
                abstract_field: self.fields.abstract_field,
                authors: self.fields.authors,
                arxiv_id: self.fields.arxiv_id,
                published_date: self.fields.published_date,
            },
        }
    }
}
