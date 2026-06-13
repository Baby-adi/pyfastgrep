use crate::{utils::*, SearchConfig};
use grep::regex::RegexMatcher;
use grep::searcher::{SearcherBuilder, Sink, SinkMatch, SinkContext};
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::path::Path;

/// A single context line with its line number.
#[derive(Clone, Debug, Serialize)]
pub struct ContextLine {
    pub line: usize,
    pub content: String,
}

/// A search hit with surrounding context lines.
#[derive(Clone, Debug, Serialize)]
pub struct SearchHitWithContext {
    pub file: String,
    pub line: usize,
    pub content: String,
    pub before_context: Vec<ContextLine>,
    pub after_context: Vec<ContextLine>,
}

/// Configuration for context search.
#[derive(Clone, Debug)]
pub struct ContextConfig {
    pub base: SearchConfig,
    pub before_context: usize,
    pub after_context: usize,
}

/// Search with context lines (-A, -B, -C equivalent).
pub fn search_with_context(config: &ContextConfig) -> Result<Vec<SearchHitWithContext>, String> {
    let matcher = build_matcher(&config.base.pattern, config.base.ignore_case, config.base.fixed_strings)?;
    let glob_matcher = build_glob(&config.base.glob)?;
    let paths = collect_paths(&config.base.root, &glob_matcher);

    let results: Vec<SearchHitWithContext> = paths
        .par_iter()
        .map(|path| {
            search_file_with_context(path, &matcher, config.before_context, config.after_context)
        })
        .flatten()
        .collect();

    Ok(results)
}

fn search_file_with_context(
    path: &Path,
    matcher: &RegexMatcher,
    before_n: usize,
    after_n: usize,
) -> Vec<SearchHitWithContext> {
    let Some(metadata) = path.metadata().ok() else {
        return Vec::new();
    };

    if metadata.len() == 0 {
        return Vec::new();
    }

    let mut sink = ContextSink::new(path.display().to_string(), before_n, after_n);
    let mut searcher = SearcherBuilder::new()
        .before_context(before_n)
        .after_context(after_n)
        .build();

    let _ = searcher.search_path(matcher, path, &mut sink);
    sink.into_hits()
}

/// Internal sink that collects matches and context lines.
struct ContextSink {
    file: String,
    before_n: usize,
    after_n: usize,
    lines: Vec<CollectedLine>,
}

#[derive(Debug)]
struct CollectedLine {
    line_number: usize,
    content: String,
    is_match: bool,
}

impl ContextSink {
    fn new(file: String, before_n: usize, after_n: usize) -> Self {
        Self {
            file,
            before_n,
            after_n,
            lines: Vec::new(),
        }
    }

    fn into_hits(self) -> Vec<SearchHitWithContext> {
        let mut hits = Vec::new();

        for (i, line) in self.lines.iter().enumerate() {
            if line.is_match {
                // Collect at most before_n context lines before this match
                let mut before = Vec::new();
                let mut j = i;
                while j > 0 && before.len() < self.before_n {
                    j -= 1;
                    if self.lines[j].is_match {
                        break;
                    }
                    before.insert(0, ContextLine {
                        line: self.lines[j].line_number,
                        content: self.lines[j].content.clone(),
                    });
                }

                // Collect at most after_n context lines after this match
                let mut after = Vec::new();
                let mut k = i + 1;
                while k < self.lines.len() && after.len() < self.after_n {
                    if self.lines[k].is_match {
                        break;
                    }
                    after.push(ContextLine {
                        line: self.lines[k].line_number,
                        content: self.lines[k].content.clone(),
                    });
                    k += 1;
                }

                hits.push(SearchHitWithContext {
                    file: self.file.clone(),
                    line: line.line_number,
                    content: line.content.clone(),
                    before_context: before,
                    after_context: after,
                });
            }
        }

        hits
    }
}

impl Sink for ContextSink {
    type Error = io::Error;

    fn matched(
        &mut self,
        _searcher: &grep::searcher::Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        let content = String::from_utf8_lossy(mat.bytes()).to_string();
        let line_number = mat.line_number().unwrap_or(0) as usize;
        self.lines.push(CollectedLine {
            line_number,
            content,
            is_match: true,
        });
        Ok(true)
    }

    fn context(
        &mut self,
        _searcher: &grep::searcher::Searcher,
        ctx: &SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        let content = String::from_utf8_lossy(ctx.bytes()).to_string();
        let line_number = ctx.line_number().unwrap_or(0) as usize;
        self.lines.push(CollectedLine {
            line_number,
            content,
            is_match: false,
        });
        Ok(true)
    }

    fn context_break(
        &mut self,
        _searcher: &grep::searcher::Searcher,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
