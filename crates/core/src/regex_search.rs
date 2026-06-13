use crate::{utils::*, SearchConfig, SearchHit};
use grep::regex::RegexMatcher;
use grep::searcher::{sinks::UTF8, SearcherBuilder};
use rayon::prelude::*;
use std::path::Path;

/// Search for a pattern in files and return all matching lines.
pub fn search(config: &SearchConfig) -> Result<Vec<SearchHit>, String> {
    let matcher = build_matcher(&config.pattern, config.ignore_case, config.fixed_strings)?;
    let glob_matcher = build_glob(&config.glob)?;
    let paths = collect_paths(&config.root, &glob_matcher);

    let mut results: Vec<SearchHit> = paths
        .par_iter()
        .map(|path| search_file(path, &matcher))
        .flatten()
        .collect();

    if let Some(max_results) = config.max_results {
        results.truncate(max_results);
    }

    Ok(results)
}

/// Count matches per file.
pub fn search_count(config: &SearchConfig) -> Result<Vec<(String, usize)>, String> {
    let matcher = build_matcher(&config.pattern, config.ignore_case, config.fixed_strings)?;
    let glob_matcher = build_glob(&config.glob)?;
    let paths = collect_paths(&config.root, &glob_matcher);

    let results: Vec<(String, usize)> = paths
        .par_iter()
        .filter_map(|path| {
            let count = count_matches_in_file(path, &matcher);
            if count > 0 {
                Some((path.display().to_string(), count))
            } else {
                None
            }
        })
        .collect();

    Ok(results)
}

/// Return filenames that contain at least one match.
pub fn search_files_with_matches(config: &SearchConfig) -> Result<Vec<String>, String> {
    let matcher = build_matcher(&config.pattern, config.ignore_case, config.fixed_strings)?;
    let glob_matcher = build_glob(&config.glob)?;
    let paths = collect_paths(&config.root, &glob_matcher);

    let results: Vec<String> = paths
        .par_iter()
        .filter_map(|path| {
            if file_has_match(path, &matcher) {
                Some(path.display().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(results)
}

/// Search for a pattern in a single file and return all matching lines.
fn search_file(path: &Path, matcher: &RegexMatcher) -> Vec<SearchHit> {
    let Some(metadata) = path.metadata().ok() else {
        return Vec::new();
    };

    if metadata.len() == 0 {
        return Vec::new();
    }

    let mut hits = Vec::new();
    let mut searcher = SearcherBuilder::new().build();

    let _ = searcher.search_path(
        matcher,
        path,
        UTF8(|lnum, line| {
            hits.push(SearchHit {
                file: path.display().to_string(),
                line: lnum as usize,
                content: line.to_string(),
            });

            Ok(true)
        }),
    );

    hits
}

/// Count total matches in a single file.
fn count_matches_in_file(path: &Path, matcher: &RegexMatcher) -> usize {
    let Some(metadata) = path.metadata().ok() else {
        return 0;
    };

    if metadata.len() == 0 {
        return 0;
    }

    let mut count = 0usize;
    let mut searcher = SearcherBuilder::new().build();

    let _ = searcher.search_path(
        matcher,
        path,
        UTF8(|_lnum, _line| {
            count += 1;
            Ok(true)
        }),
    );

    count
}

/// Check if a file contains at least one match.
fn file_has_match(path: &Path, matcher: &RegexMatcher) -> bool {
    let Some(metadata) = path.metadata().ok() else {
        return false;
    };

    if metadata.len() == 0 {
        return false;
    }

    let mut found = false;
    let mut searcher = SearcherBuilder::new().build();

    let _ = searcher.search_path(
        matcher,
        path,
        UTF8(|_lnum, _line| {
            found = true;
            Ok(false) // Stop immediately after first match
        }),
    );

    found
}
