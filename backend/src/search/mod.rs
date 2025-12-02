//! Tantivy full-text search module for papers.

pub mod index;
pub mod query;
pub mod schema;

pub use index::SearchIndex;
pub use query::{SearchParams, SearchResponse, SearchFacets, DateBucket};
pub use schema::create_paper_schema;
