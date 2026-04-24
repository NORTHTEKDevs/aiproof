use aiproof_core::document::{Document, Kind, PromptText, Role};
use logos::Logos;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

#[derive(logos::Logos, Debug, PartialEq)]
pub(crate) enum Tok {
    #[regex(r"\{\{\{[^}]*\}\}\}")]
    TripleVar,
    #[regex(r"\{\{![^}]*\}\}")]
    Comment,
    #[regex(r"\{\{#[^}]*\}\}")]
    SectionOpen,
    #[regex(r"\{\{\^[^}]*\}\}")]
    Inverted,
    #[regex(r"\{\{/[^}]*\}\}")]
    SectionClose,
    #[regex(r"\{\{>[^}]*\}\}")]
    Partial,
    #[regex(r"\{\{&[^}]*\}\}")]
    Unescaped,
    #[regex(r"\{\{[^}]*\}\}")]
    Var,
    #[regex(r"[^{]+")]
    Text,
    #[regex(r"\{")]
    Brace,
}

static IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());

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
        kind: Kind::Mustache { variables },
    }])
}

fn extract_variables(source: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut ordered = Vec::new();
    let mut lex = Tok::lexer(source);

    while let Some(tok) = lex.next() {
        match tok {
            Ok(Tok::Var) | Ok(Tok::TripleVar) | Ok(Tok::Unescaped) | Ok(Tok::SectionOpen)
            | Ok(Tok::Inverted) | Ok(Tok::Partial) => {
                if let Some(m) = IDENT.find(lex.slice()) {
                    let ident = m.as_str();
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
    fn captures_variable_and_section() {
        let src = "Hello {{name}}!\n{{#items}}- {{title}}\n{{/items}}";
        let docs = parse(std::path::Path::new("t.mustache"), src).unwrap();
        match &docs[0].kind {
            Kind::Mustache { variables } => {
                assert!(variables.contains(&"name".to_string()));
                assert!(variables.contains(&"items".to_string()));
                assert!(variables.contains(&"title".to_string()));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn ignores_comments_and_close_tags() {
        let src = "{{! skipthis }}{{#outer}}X{{/outer}}{{ shown }}";
        let docs = parse(std::path::Path::new("t.mustache"), src).unwrap();
        match &docs[0].kind {
            Kind::Mustache { variables } => {
                assert!(variables.contains(&"shown".to_string()));
                assert!(variables.contains(&"outer".to_string()));
                assert!(!variables.iter().any(|v| v == "skipthis"));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn triple_stache_captured() {
        let src = "{{{ raw_html }}}";
        let docs = parse(std::path::Path::new("t.mustache"), src).unwrap();
        match &docs[0].kind {
            Kind::Mustache { variables } => {
                assert_eq!(variables, &vec!["raw_html".to_string()]);
            }
            _ => panic!(),
        }
    }
}
