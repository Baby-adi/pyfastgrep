use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;

pub fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn hit_to_csv_row(file: &str, line: usize, content: &str) -> String {
    format!(
        "{},{},{}\n",
        csv_escape(file),
        line,
        csv_escape(content.trim_end())
    )
}

pub fn hits_to_csv_header() -> String {
    String::from("file,line,content\n")
}

pub fn count_results_to_csv(results: &[(String, usize)]) -> String {
    let mut output = String::from("file,count\n");
    for (file, count) in results {
        output.push_str(&format!("{},{}\n", csv_escape(file), count));
    }
    output
}

pub fn write_csv_file(path: &str, csv_content: &str) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(csv_content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hits_to_json_string(hits: &[(String, usize, String)]) -> String {
    let json_results: Vec<Value> = hits
        .iter()
        .map(|(file, line, content)| {
            json!({
                "file": file,
                "line": line,
                "content": content.trim_end()
            })
        })
        .collect();

    serde_json::to_string(&json_results).unwrap()
}
