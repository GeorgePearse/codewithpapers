//! Tantivy schema definition for papers.

use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED,
    STRING,
};

/// Field names for the paper index
pub struct PaperFields {
    pub id: Field,
    pub title: Field,
    pub abstract_field: Field,
    pub authors: Field,
    pub arxiv_id: Field,
    pub published_date: Field,
}

/// Create the Tantivy schema for papers.
pub fn create_paper_schema() -> (Schema, PaperFields) {
    let mut schema_builder = Schema::builder();

    // Stored ID for fetching full paper from PostgreSQL
    let id = schema_builder.add_text_field("id", STRING | STORED);

    // Full-text searchable fields with English stemming
    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("en_stem")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    let title = schema_builder.add_text_field("title", text_options.clone());
    let abstract_field = schema_builder.add_text_field("abstract", text_options.clone());
    let authors = schema_builder.add_text_field("authors", text_options);

    // Exact match field for arxiv_id
    let arxiv_id = schema_builder.add_text_field("arxiv_id", STRING | STORED);

    // Date field for faceted search (FAST enables efficient range queries)
    let published_date = schema_builder.add_date_field("published_date", INDEXED | STORED | FAST);

    let schema = schema_builder.build();

    let fields = PaperFields {
        id,
        title,
        abstract_field,
        authors,
        arxiv_id,
        published_date,
    };

    (schema, fields)
}
