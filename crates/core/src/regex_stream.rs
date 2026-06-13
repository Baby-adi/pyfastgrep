use crate::{utils::*, SearchConfig, SearchHit, SearchReceiver};
use crossbeam_channel::bounded;
use grep::searcher::{sinks::UTF8, SearcherBuilder};
use ignore::WalkBuilder;
use std::thread;

/// Streaming search that returns a channel receiver.
pub fn search_stream(config: SearchConfig) -> Result<SearchReceiver, String> {
    let matcher = build_matcher(&config.pattern, config.ignore_case, config.fixed_strings)?;
    let glob_matcher = build_glob(&config.glob)?;
    let (tx, rx) = bounded(1000);

    thread::spawn(move || {
        let walker = WalkBuilder::new(&config.root)
            .standard_filters(true)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
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

            let path = entry.path().to_path_buf();

            if path.metadata().map(|m| m.len() == 0).unwrap_or(false) {
                continue;
            }

            let mut searcher = SearcherBuilder::new().build();

            let _ = searcher.search_path(
                &matcher,
                &path,
                UTF8(|lnum, line| {
                    if tx
                        .send(SearchHit {
                            file: path.display().to_string(),
                            line: lnum as usize,
                            content: line.to_string(),
                        })
                        .is_err()
                    {
                        return Ok(false);
                    }

                    Ok(true)
                }),
            );
        }
    });

    Ok(rx)
}
