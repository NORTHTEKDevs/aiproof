use aiproof_core::document::{Document, Kind, PromptText, Role};
use std::path::Path;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    // Detect Prompty frontmatter.
    if let Some((_, body)) = split_prompty_frontmatter(source) {
        return Ok(vec![Document {
            path: path.to_path_buf(),
            role: Role::System,
            source: source.to_string(),
            prompt: PromptText {
                text: body.to_string(),
                origin_span: None,
            },
            kind: Kind::YamlConfig,
        }]);
    }
    Ok(vec![Document {
        path: path.to_path_buf(),
        role: Role::Unknown,
        source: source.to_string(),
        prompt: PromptText {
            text: source.to_string(),
            origin_span: None,
        },
        kind: Kind::YamlConfig,
    }])
}

fn split_prompty_frontmatter(src: &str) -> Option<(&str, &str)> {
    let s = src.strip_prefix("---\n")?;
    let end = s.find("\n---\n")?;
    let frontmatter = &s[..end];
    let body = &s[end + "\n---\n".len()..];
    Some((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompty_extracts_body() {
        let src = "---\nname: test\nmodel: claude-4.7-opus\n---\nYou are a helpful assistant.\n";
        let docs = parse(std::path::Path::new("p.yaml"), src).unwrap();
        assert_eq!(docs[0].prompt.text.trim(), "You are a helpful assistant.");
        assert_eq!(docs[0].role, Role::System);
    }

    #[test]
    fn plain_yaml_keeps_full_source() {
        let src = "foo: bar\nbaz: qux\n";
        let docs = parse(std::path::Path::new("c.yaml"), src).unwrap();
        assert_eq!(docs[0].prompt.text, src);
        assert_eq!(docs[0].role, Role::Unknown);
    }
}
