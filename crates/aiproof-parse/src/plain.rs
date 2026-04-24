use aiproof_core::document::{Document, Kind, PromptText, Role};
use std::path::Path;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    Ok(vec![Document {
        path: path.to_path_buf(),
        role: Role::Unknown,
        source: source.to_string(),
        prompt: PromptText {
            text: source.to_string(),
            origin_span: None,
        },
        kind: Kind::PlainText,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_roundtrips_source() {
        let docs = parse(std::path::Path::new("a.prompt"), "hello").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].prompt.text, "hello");
    }
}
