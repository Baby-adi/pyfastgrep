use serde::Serialize;
use std::path::PathBuf;

pub mod ast;
pub mod ast_search;
pub mod context_search;
pub mod regex_search;
pub mod regex_stream;
pub mod utils;

/// Configuration for a search operation.
#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub pattern: String,
    pub root: PathBuf,
    pub glob: Option<String>,
    pub max_results: Option<usize>,
    pub ignore_case: bool,
    pub fixed_strings: bool,
    pub byte_offset: bool,
}

impl SearchConfig {
    pub fn new(pattern: impl Into<String>, root: impl Into<PathBuf>) -> Self {
        Self {
            pattern: pattern.into(),
            root: root.into(),
            glob: None,
            max_results: None,
            ignore_case: false,
            fixed_strings: false,
            byte_offset: false,
        }
    }
}

/// A single search result hit.
#[derive(Clone, Debug, Serialize)]
pub struct SearchHit {
    pub file: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_offset: Option<u64>,
    pub content: String,
}

/// Receiver for streaming search results.
pub type SearchReceiver = crossbeam_channel::Receiver<SearchHit>;

// Re-export regex search functions
pub use regex_search::{search, search_count, search_files_with_matches};

// Re-export streaming search
pub use regex_stream::search_stream;

// Re-export AST search
pub use ast_search::{search_ast, search_ast_stream, AstQueryType, AstResultReceiver};

// Re-export context search
pub use context_search::{search_with_context, ContextConfig, SearchHitWithContext};
