use aiproof_core::document::{Document, Kind, PromptText, Role};
use std::path::Path;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(source) else {
        // Invalid JSON — fall back to plain.
        return crate::plain::parse(path, source);
    };

    let mut descriptions = Vec::new();
    collect_descriptions(&value, &mut descriptions);

    if descriptions.is_empty() {
        return crate::plain::parse(path, source);
    }

    Ok(descriptions
        .into_iter()
        .map(|d| Document {
            path: path.to_path_buf(),
            role: Role::Tool,
            source: source.to_string(),
            prompt: PromptText {
                text: d,
                origin_span: None,
            },
            kind: Kind::JsonSchema,
        })
        .collect())
}

fn collect_descriptions(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(d)) = map.get("description") {
                out.push(d.clone());
            }
            for (_, child) in map {
                collect_descriptions(child, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr {
                collect_descriptions(child, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mcp_tool_descriptions() {
        let src = r#"{
            "name": "search",
            "description": "Search the knowledge base for a term.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search term." }
                }
            }
        }"#;
        let docs = parse(std::path::Path::new("t.json"), src).unwrap();
        assert_eq!(docs.len(), 2);
        let texts: Vec<_> = docs.iter().map(|d| d.prompt.text.as_str()).collect();
        assert!(texts.contains(&"Search the knowledge base for a term."));
        assert!(texts.contains(&"The search term."));
    }

    #[test]
    fn non_mcp_json_falls_back_to_plain() {
        let src = r#"{"foo": 1}"#;
        let docs = parse(std::path::Path::new("d.json"), src).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].prompt.text, src);
    }

    #[test]
    fn invalid_json_falls_back_to_plain() {
        let src = "not json at all";
        let docs = parse(std::path::Path::new("x.json"), src).unwrap();
        assert_eq!(docs[0].prompt.text, "not json at all");
    }
}
