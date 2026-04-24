//! aiproof-parse: per-format parsers producing Document values.

pub mod jinja;
pub mod json_schema;
pub mod markdown;
pub mod mustache;
pub mod plain;
pub mod python_extract;
pub mod ts_extract;
pub mod yaml;

use aiproof_core::document::Document;
use std::path::Path;

pub fn parse_file(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "md" => markdown::parse(path, source),
        "j2" | "jinja" | "jinja2" => jinja::parse(path, source),
        "mustache" => mustache::parse(path, source),
        "yaml" | "yml" => yaml::parse(path, source),
        "json" => json_schema::parse(path, source),
        "py" => python_extract::parse(path, source),
        "ts" | "tsx" => ts_extract::parse(path, source),
        _ => plain::parse(path, source),
    }
}
