use pyfastgrep_core::{
    search, search_ast, search_count, search_files_with_matches, search_with_context, AstQueryType,
    ContextConfig, SearchConfig, SearchHit,
};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn hit_to_csv_row(hit: &SearchHit) -> String {
    format!(
        "{},{},{}\n",
        csv_escape(&hit.file),
        hit.line,
        csv_escape(hit.content.trim_end())
    )
}

fn hits_to_csv(hits: &[SearchHit]) -> String {
    let mut output = String::from("file,line,content\n");

    for hit in hits {
        output.push_str(&hit_to_csv_row(hit));
    }

    output
}

fn count_results_to_csv(results: &[(String, usize)]) -> String {
    let mut output = String::from("file,count\n");
    for (file, count) in results {
        output.push_str(&format!("{},{}\n", csv_escape(file), count));
    }
    output
}

fn write_csv_file(path: &str, csv_content: &str) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(csv_content.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn print_regular_hit(hit: &SearchHit) {
    if let Some(offset) = hit.byte_offset {
        println!(
            "{}:{}:{}: {}",
            hit.file,
            offset,
            hit.line,
            hit.content.trim_end()
        );
    } else {
        println!("{}:{}: {}", hit.file, hit.line, hit.content.trim_end());
    }
}

macro_rules! print_json {
    ($value:expr) => {
        match serde_json::to_string_pretty($value) {
            Ok(json_str) => println!("{}", json_str),
            Err(err) => {
                eprintln!("Error serializing to JSON: {err}");
                std::process::exit(1);
            }
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        std::process::exit(1);
    }

    let mut pattern: Option<String> = None;
    let mut root = PathBuf::from(".");
    let mut glob: Option<String> = None;
    let mut max_results: Option<usize> = None;
    let mut ignore_case = false;
    let mut json = false;
    let mut csv = false;
    let mut output_path: Option<String> = None;
    let mut count = false;
    let mut files_with_matches = false;
    let mut fixed_strings = false;
    let mut byte_offset = false;
    let mut before_context: Option<usize> = None;
    let mut after_context: Option<usize> = None;
    let mut context: Option<usize> = None;
    let mut ast_mode: Option<AstQueryType> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-g" | "--glob" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --glob");
                    std::process::exit(1);
                }
                glob = Some(args[i].clone());
            }
            "-n" | "--limit" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --limit");
                    std::process::exit(1);
                }
                max_results = args[i].parse::<usize>().ok();
            }
            "-i" | "--ignore-case" => {
                ignore_case = true;
            }
            "-j" | "--json" => {
                json = true;
            }
            "--csv" => {
                csv = true;
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --output");
                    std::process::exit(1);
                }
                output_path = Some(args[i].clone());
            }
            "-r" | "--root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --root");
                    std::process::exit(1);
                }
                root = PathBuf::from(&args[i]);
            }
            "-c" | "--count" => {
                count = true;
            }
            "-l" | "--files-with-matches" => {
                files_with_matches = true;
            }
            "-F" | "--fixed-strings" => {
                fixed_strings = true;
            }
            "-b" | "--byte-offset" => {
                byte_offset = true;
            }
            "-A" | "--after-context" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --after-context");
                    std::process::exit(1);
                }
                after_context = args[i].parse::<usize>().ok();
            }
            "-B" | "--before-context" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --before-context");
                    std::process::exit(1);
                }
                before_context = args[i].parse::<usize>().ok();
            }
            "-C" | "--context" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing value for --context");
                    std::process::exit(1);
                }
                context = args[i].parse::<usize>().ok();
            }
            "--functions" => {
                ast_mode = Some(AstQueryType::Function);
            }
            "--classes" => {
                ast_mode = Some(AstQueryType::Class);
            }
            "--imports" => {
                ast_mode = Some(AstQueryType::Import);
            }
            value if value.starts_with('-') => {
                eprintln!("Unknown flag: {value}");
                print_usage();
                std::process::exit(1);
            }
            value => {
                if pattern.is_none() {
                    pattern = Some(value.to_string());
                } else if root == PathBuf::from(".") {
                    root = PathBuf::from(value);
                } else {
                    eprintln!("Unexpected positional argument: {value}");
                    print_usage();
                    std::process::exit(1);
                }
            }
        }

        i += 1;
    }

    let Some(pattern) = pattern else {
        eprintln!("Missing search pattern");
        print_usage();
        std::process::exit(1);
    };

    // Resolve context values: -C sets both -A and -B
    let resolved_before = before_context.or(context).unwrap_or(0);
    let resolved_after = after_context.or(context).unwrap_or(0);
    let has_context = resolved_before > 0 || resolved_after > 0;

    // Mutually exclusive format flags
    let format_flags = [("--json", json), ("--csv", csv)];
    let active_formats: Vec<&str> = format_flags
        .iter()
        .filter(|(_, active)| *active)
        .map(|(name, _)| *name)
        .collect();
    if active_formats.len() > 1 {
        eprintln!(
            "Error: {} are mutually exclusive",
            active_formats.join(", ")
        );
        std::process::exit(1);
    }

    // Mutually exclusive mode flags
    let mode_flags = [
        ("--count", count),
        ("--files-with-matches", files_with_matches),
    ];
    let active_modes: Vec<&str> = mode_flags
        .iter()
        .filter(|(_, active)| *active)
        .map(|(name, _)| *name)
        .collect();
    if active_modes.len() > 1 {
        eprintln!("Error: {} are mutually exclusive", active_modes.join(", "));
        std::process::exit(1);
    }

    if output_path.is_some() && !csv {
        eprintln!("Error: --output is only supported with --csv");
        std::process::exit(1);
    }

    if has_context && (count || files_with_matches || ast_mode.is_some()) {
        eprintln!(
            "Error: --context is not supported with --count, --files-with-matches, or AST search"
        );
        std::process::exit(1);
    }

    if count || files_with_matches {
        let mut config = SearchConfig::new(pattern, root);
        config.glob = glob;
        config.ignore_case = ignore_case;
        config.fixed_strings = fixed_strings;

        if count {
            match search_count(&config) {
                Ok(results) => {
                    if json {
                        let json_results: Vec<serde_json::Value> = results
                            .into_iter()
                            .map(|(file, count)| {
                                serde_json::json!({
                                    "file": file,
                                    "count": count,
                                })
                            })
                            .collect();
                        print_json!(&json_results);
                    } else if csv {
                        let csv_output = count_results_to_csv(&results);
                        if let Some(path) = output_path.as_deref() {
                            if let Err(err) = write_csv_file(path, &csv_output) {
                                eprintln!("Error writing CSV output: {err}");
                                std::process::exit(1);
                            }
                        }
                        print!("{}", csv_output);
                    } else {
                        for (file, count) in results {
                            println!("{}:{}", file, count);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::exit(1);
                }
            }
        } else {
            match search_files_with_matches(&config) {
                Ok(results) => {
                    if json {
                        print_json!(&results);
                    } else if csv {
                        let mut output = String::from("file\n");
                        for file in &results {
                            output.push_str(&format!("{}\n", csv_escape(file)));
                        }
                        if let Some(path) = output_path.as_deref() {
                            if let Err(err) = write_csv_file(path, &output) {
                                eprintln!("Error writing CSV output: {err}");
                                std::process::exit(1);
                            }
                        }
                        print!("{}", output);
                    } else {
                        for file in results {
                            println!("{}", file);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {err}");
                    std::process::exit(1);
                }
            }
        }
    } else if let Some(query_type) = ast_mode {
        // AST search
        match search_ast(&pattern, &root, &glob, query_type) {
            Ok(results) => {
                let hits: Vec<SearchHit> = results
                    .into_iter()
                    .map(|(file, line, content)| SearchHit {
                        file,
                        line,
                        byte_offset: None,
                        content,
                    })
                    .collect();

                if json {
                    print_json!(&hits);
                } else if csv {
                    let csv_output = hits_to_csv(&hits);
                    if let Some(path) = output_path.as_deref() {
                        if let Err(err) = write_csv_file(path, &csv_output) {
                            eprintln!("Error writing CSV output: {err}");
                            std::process::exit(1);
                        }
                    }
                    print!("{}", csv_output);
                } else {
                    for hit in hits {
                        print_regular_hit(&hit);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
    } else if has_context {
        // Context search
        let mut config = ContextConfig {
            base: SearchConfig::new(pattern, root),
            before_context: resolved_before,
            after_context: resolved_after,
        };
        config.base.glob = glob;
        config.base.ignore_case = ignore_case;
        config.base.fixed_strings = fixed_strings;
        config.base.max_results = max_results;

        match search_with_context(&config) {
            Ok(results) => {
                if json {
                    match serde_json::to_string_pretty(&results) {
                        Ok(json_str) => println!("{}", json_str),
                        Err(err) => {
                            eprintln!("Error serializing JSON: {err}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    for hit in results {
                        // Print before context
                        for (idx, line) in hit.before_context.iter().enumerate() {
                            let ctx_line = hit.line - hit.before_context.len() + idx;
                            println!("{}-{}-{}", hit.file, ctx_line, line.content.trim_end());
                        }
                        // Print match
                        println!("{}:{}:{}", hit.file, hit.line, hit.content.trim_end());
                        // Print after context
                        for (idx, line) in hit.after_context.iter().enumerate() {
                            let ctx_line = hit.line + 1 + idx;
                            println!("{}-{}-{}", hit.file, ctx_line, line.content.trim_end());
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
    } else {
        // Regular regex search
        let mut config = SearchConfig::new(pattern, root);
        config.glob = glob;
        config.max_results = max_results;
        config.ignore_case = ignore_case;
        config.fixed_strings = fixed_strings;
        config.byte_offset = byte_offset;

        match search(&config) {
            Ok(results) => {
                if json {
                    print_json!(&results);
                } else if csv {
                    let csv_output = hits_to_csv(&results);
                    if let Some(path) = output_path.as_deref() {
                        if let Err(err) = write_csv_file(path, &csv_output) {
                            eprintln!("Error writing CSV output: {err}");
                            std::process::exit(1);
                        }
                    }
                    print!("{}", csv_output);
                } else {
                    for hit in results {
                        print_regular_hit(&hit);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {err}");
                std::process::exit(1);
            }
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage: pyfastgrep <pattern> [root] [--glob <pattern>] [--limit <n>] [--ignore-case] [--fixed-strings] [--byte-offset] [--json] [--csv] [--output <file>] [--root <path>] [--count] [--files-with-matches] [--context <n>] [--before-context <n>] [--after-context <n>] [--functions] [--classes] [--imports]"
    );
}
