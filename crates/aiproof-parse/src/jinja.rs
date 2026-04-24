use aiproof_core::document::{Document, Kind, PromptText, Role};
use logos::Logos;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

#[derive(logos::Logos, Debug, PartialEq)]
pub(crate) enum Tok {
    #[regex(r"\{\{[^}]*\}\}")]
    Expr,
    #[regex(r"\{%[^%]*%\}")]
    Stmt,
    #[regex(r"\{#[^#]*#\}")]
    Comment,
    #[regex(r"[^{]+")]
    Text,
    #[regex(r"\{")]
    Brace,
}

static IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());

const KEYWORDS: &[&str] = &[
    "for", "in", "if", "not", "and", "or", "true", "false", "none", "null", "endfor", "endif",
    "else", "elif", "set", "with", "endwith", "block", "endblock", "extends", "include", "macro",
    "endmacro", "is", "as", "do", "from", "import",
];

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let variables = extract_variables(source);
    Ok(vec![Document {
        path: path.to_path_buf(),
        role: Role::Unknown,
        source: source.to_string(),
        prompt: PromptText {
            text: source.to_string(),
            origin_span: None,
        },
        kind: Kind::Jinja { variables },
    }])
}

fn extract_variables(source: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered = Vec::new();
    let mut lex = Tok::lexer(source);

    while let Some(tok) = lex.next() {
        match tok {
            Ok(Tok::Expr) | Ok(Tok::Stmt) => {
                for m in IDENT.find_iter(lex.slice()) {
                    let ident = m.as_str();
                    if KEYWORDS.contains(&ident) {
                        continue;
                    }
                    if seen.insert(ident.to_string()) {
                        ordered.push(ident.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    ordered
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::document::Kind;

    #[test]
    fn captures_expression_variables() {
        let src = "Hello {{ name }}, your order #{{ order_id }} is ready.";
        let docs = parse(std::path::Path::new("t.j2"), src).unwrap();
        match &docs[0].kind {
            Kind::Jinja { variables } => {
                assert_eq!(variables, &vec!["name".to_string(), "order_id".to_string()]);
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn captures_loop_variables_skipping_keywords() {
        let src = "{% for item in items %}- {{ item }}\n{% endfor %}";
        let docs = parse(std::path::Path::new("t.j2"), src).unwrap();
        match &docs[0].kind {
            Kind::Jinja { variables } => {
                assert!(variables.contains(&"item".to_string()));
                assert!(variables.contains(&"items".to_string()));
                assert!(!variables.contains(&"for".to_string()));
                assert!(!variables.contains(&"in".to_string()));
                assert!(!variables.contains(&"endfor".to_string()));
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn ignores_comments() {
        let src = "{# this mentions secretname inside a comment #}{{ shown }}";
        let docs = parse(std::path::Path::new("t.j2"), src).unwrap();
        match &docs[0].kind {
            Kind::Jinja { variables } => {
                assert_eq!(variables, &vec!["shown".to_string()]);
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn deduplicates_preserving_first_appearance() {
        let src = "{{ a }}{{ b }}{{ a }}{{ c }}";
        let docs = parse(std::path::Path::new("t.j2"), src).unwrap();
        match &docs[0].kind {
            Kind::Jinja { variables } => {
                assert_eq!(
                    variables,
                    &vec!["a".to_string(), "b".to_string(), "c".to_string()]
                );
            }
            _ => panic!("wrong kind"),
        }
    }
}
