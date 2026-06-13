use globset::{Glob, GlobSet, GlobSetBuilder};
use grep::regex::{RegexMatcher, RegexMatcherBuilder};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Build a glob matcher from an optional glob string.
pub fn build_glob(glob: &Option<String>) -> Result<Option<GlobSet>, String> {
    if let Some(g) = glob {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new(g).map_err(|e| e.to_string())?);
        Ok(Some(builder.build().map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

/// Build a regex matcher from a pattern string.
pub fn build_matcher(
    pattern: &str,
    ignore_case: bool,
    fixed_strings: bool,
) -> Result<RegexMatcher, String> {
    RegexMatcherBuilder::new()
        .case_insensitive(ignore_case)
        .fixed_strings(fixed_strings)
        .build(pattern)
        .map_err(|e| e.to_string())
}

/// Collect all file paths matching the given root and glob pattern.
pub fn collect_paths(root: &Path, glob_matcher: &Option<GlobSet>) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .standard_filters(true)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|entry| match glob_matcher {
            Some(gs) => gs.is_match(entry.path()),
            None => true,
        })
        .map(|entry| entry.into_path())
        .collect()
}
