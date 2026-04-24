use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::span::Span;
use std::path::Path;
use tree_sitter::Parser;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    // Verify tree-sitter-md accepts the source (future rules will walk the AST).
    // API: tree_sitter_md::language() and tree_sitter_md::inline_language() for block/inline grammars.
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_md::language())?;
    let _tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter-md failed to parse"))?;
    // TODO: expose AST to rules when needed

    let (role, body_offset) = extract_frontmatter(source);
    let body = &source[body_offset..];
    let trimmed = body.trim_start();
    let trim_offset = body_offset + (body.len() - trimmed.len());
    let origin_span = Span::from_byte_range(source, trim_offset..source.len());

    Ok(vec![Document {
        path: path.to_path_buf(),
        role,
        source: source.to_string(),
        prompt: PromptText {
            text: trimmed.trim_end().to_string(),
            origin_span: Some(origin_span),
        },
        kind: Kind::Markdown,
    }])
}

/// Returns (role, byte offset at which body content starts).
fn extract_frontmatter(src: &str) -> (Role, usize) {
    let Some(rest) = src.strip_prefix("---\n") else {
        return (Role::Unknown, 0);
    };
    let Some(end) = rest.find("\n---\n") else {
        return (Role::Unknown, 0);
    };
    let frontmatter = &rest[..end];
    let body_offset = "---\n".len() + end + "\n---\n".len();
    let role = frontmatter
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(k, _)| k.trim() == "role")
        .map(|(_, v)| v.trim().trim_matches('"').trim_matches('\''))
        .and_then(|v| match v {
            "system" => Some(Role::System),
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            "tool" => Some(Role::Tool),
            _ => None,
        })
        .unwrap_or(Role::Unknown);
    (role, body_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_system_role_and_body() {
        let src = "---\nrole: system\n---\nYou are a helpful assistant.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].role, Role::System);
        insta::assert_yaml_snapshot!(docs[0].prompt.text);
    }

    #[test]
    fn no_frontmatter_whole_body_is_prompt() {
        let src = "# Prompt\n\nAnswer concisely.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs[0].role, Role::Unknown);
        insta::assert_yaml_snapshot!(docs[0].prompt.text);
    }

    #[test]
    fn malformed_frontmatter_treated_as_body() {
        // No closing --- fence.
        let src = "---\nrole: system\nYou are helpful.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs[0].role, Role::Unknown);
        insta::assert_yaml_snapshot!(docs[0].prompt.text);
    }

    #[test]
    fn unknown_role_falls_back() {
        let src = "---\nrole: sidekick\n---\nHi.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs[0].role, Role::Unknown);
    }

    #[test]
    fn quoted_role_value() {
        let src = "---\nrole: \"user\"\n---\nTest message.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs[0].role, Role::User);
        insta::assert_yaml_snapshot!(docs[0].prompt.text);
    }

    #[test]
    fn multiple_frontmatter_keys() {
        let src = "---\nname: test\nrole: assistant\nmodel: gpt-4\n---\nAssistant response.\n";
        let docs = parse(Path::new("x.md"), src).unwrap();
        assert_eq!(docs[0].role, Role::Assistant);
        insta::assert_yaml_snapshot!(docs[0].prompt.text);
    }
}
