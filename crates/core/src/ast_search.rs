use crate::utils::*;
use crate::ast::TargetLanguage;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{bounded, Receiver};
use ignore::WalkBuilder;
use tree_sitter::{Parser, Query, QueryCursor};

/// Type of AST query to perform.
#[derive(Clone, Copy, Debug)]
pub enum AstQueryType {
    Function,
    Class,
    Import,
}

/// Receiver for streaming AST search results.
pub type AstResultReceiver = Receiver<(String, usize, String)>;

/// Search for AST nodes by name.
pub fn search_ast(
    target_name: &str,
    root: &Path,
    glob: &Option<String>,
    query_type: AstQueryType,
) -> Result<Vec<(String, usize, String)>, String> {
    let glob_matcher = build_glob(glob)?;
    let results = Arc::new(Mutex::new(Vec::new()));

    let paths = collect_paths(root, &glob_matcher);

    paths.par_iter().for_each(|path| {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if let Some(lang) = TargetLanguage::from_extension(ext) {
            if let Ok(source_code) = fs::read_to_string(path) {
                let mut parser = Parser::new();
                let ts_lang = lang.get_parser_language();
                let _ = parser.set_language(ts_lang);

                if let Some(tree) = parser.parse(&source_code, None) {
                    let query_str = match query_type {
                        AstQueryType::Function => lang.function_query(),
                        AstQueryType::Class => lang.class_query(),
                        AstQueryType::Import => lang.import_query(),
                    };

                    if let Ok(query) = Query::new(ts_lang, query_str) {
                        let mut cursor = QueryCursor::new();
                        let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

                        for m in matches {
                            for capture in m.captures {
                                let node = capture.node;
                                let node_text = &source_code[node.byte_range()];

                                let is_match = match query_type {
                                    AstQueryType::Import => node_text.contains(target_name),
                                    _ => node_text == target_name,
                                };

                                if is_match {
                                    let start_pos = node.start_position();
                                    let line = source_code.lines().nth(start_pos.row).unwrap_or("").to_string();
                                    let mut res = results.lock().unwrap();
                                    let item = (path.display().to_string(), start_pos.row + 1, line);
                                    if !res.contains(&item) {
                                        res.push(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let final_results = Arc::try_unwrap(results)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok(final_results)
}

/// Streaming AST search.
pub fn search_ast_stream(
    target_name: String,
    root: String,
    glob: Option<String>,
    query_type: AstQueryType,
) -> Result<AstResultReceiver, String> {
    let glob_matcher = build_glob(&glob)?;
    let (tx, rx) = bounded(1000);

    thread::spawn(move || {
        let walker = WalkBuilder::new(&root)
            .standard_filters(true)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }

            if let Some(ref gs) = glob_matcher {
                if !gs.is_match(entry.path()) {
                    continue;
                }
            }

            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if let Some(lang) = TargetLanguage::from_extension(ext) {
                if let Ok(source_code) = fs::read_to_string(path) {
                    let mut parser = Parser::new();
                    let ts_lang = lang.get_parser_language();
                    let _ = parser.set_language(ts_lang);

                    if let Some(tree) = parser.parse(&source_code, None) {
                        let query_str = match query_type {
                            AstQueryType::Function => lang.function_query(),
                            AstQueryType::Class => lang.class_query(),
                            AstQueryType::Import => lang.import_query(),
                        };

                        if let Ok(query) = Query::new(ts_lang, query_str) {
                            let mut cursor = QueryCursor::new();
                            let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

                            for m in matches {
                                for capture in m.captures {
                                    let node = capture.node;
                                    let node_text = &source_code[node.byte_range()];

                                    let is_match = match query_type {
                                        AstQueryType::Import => node_text.contains(&target_name),
                                        _ => node_text == target_name,
                                    };

                                    if is_match {
                                        let start_pos = node.start_position();
                                        let line = source_code.lines().nth(start_pos.row).unwrap_or("").to_string();
                                        if tx.send((path.display().to_string(), start_pos.row + 1, line)).is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(rx)
}
