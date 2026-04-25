//! AIP012: unused-template-variable
//!
//! Detect unused template variables, focusing on Jinja {% set %} declarations for v0.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic, Edit, Fix},
    document::{Document, Kind},
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

static SET_DECL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{%\s*set\s+([A-Za-z_]\w*)\s*=").unwrap());

pub struct UnusedTemplateVariable;

impl Rule for UnusedTemplateVariable {
    fn code(&self) -> &'static str {
        "AIP012"
    }

    fn name(&self) -> &'static str {
        "unused-template-variable"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Only check Jinja templates for v0.
        let Kind::Jinja { variables: _ } = &doc.kind else {
            return diags;
        };

        let text = &doc.prompt.text;

        for m in SET_DECL.captures_iter(text) {
            if let (Some(full), Some(ident_match)) = (m.get(0), m.get(1)) {
                let var_name = ident_match.as_str();
                let decl_span = full.range();

                // Count references to this variable outside the declaration.
                // References are: {{ var_name }} or {% ... var_name ... %}
                if !has_references(text, var_name, decl_span.clone()) {
                    // Find the full line to remove (from start to end of line).
                    let line_span = find_line_span(text, decl_span);
                    let span = project_span(doc, line_span);

                    diags.push(Diagnostic {
                        code: "AIP012".to_string(),
                        message: format!("variable '{}' is declared but never used", var_name),
                        severity: Severity::Info,
                        category: Category::Efficiency,
                        primary: span,
                        labels: vec![],
                        explain_url: Some("https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP012.md".to_string()),
                        fix: None,
                    });
                }
            }
        }

        diags
    }

    fn autofix(&self, diag: &Diagnostic, _doc: &Document) -> Option<Fix> {
        Some(Fix {
            description: "remove the unused {% set %} declaration".into(),
            edits: vec![Edit {
                span: diag.primary.clone(),
                replacement: String::new(),
            }],
            safe: true,
        })
    }
}

/// Check if a variable has any references outside its declaration.
fn has_references(text: &str, var_name: &str, decl_span: std::ops::Range<usize>) -> bool {
    // Search for the variable name in the text, excluding the declaration.
    // Look for word-boundary matches (to avoid partial matches like "total" in "totally").

    // Build a regex that matches the variable name as a complete word.
    let escaped = regex::escape(var_name);
    let word_pattern = format!(r"\b{}\b", escaped);

    if let Ok(re) = Regex::new(&word_pattern) {
        for m in re.find_iter(text) {
            // Skip if this match is within the declaration span.
            if decl_span.contains(&m.start()) {
                continue;
            }
            // Found a reference outside the declaration.
            return true;
        }
    }

    false
}

/// Find the span of the line containing the given position.
/// Includes the newline at the end if present.
fn find_line_span(text: &str, pos_range: std::ops::Range<usize>) -> std::ops::Range<usize> {
    let start = pos_range.start;

    // Find the start of the line.
    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);

    // Find the end of the line.
    let line_end = text[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(text.len());

    line_start..line_end
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(UnusedTemplateVariable));
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::document::{Kind, PromptText, Role};
    use std::path::PathBuf;

    #[test]
    fn unused_jinja_set_detected() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "{% set total = 5 %}\nHello world.".to_string(),
                origin_span: None,
            },
            kind: Kind::Jinja {
                variables: vec!["total".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("never used"));
    }

    #[test]
    fn used_jinja_variable_not_flagged() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "{% set total = 5 %}\nCount is {{total}}".to_string(),
                origin_span: None,
            },
            kind: Kind::Jinja {
                variables: vec!["total".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn autofix_removes_set_line() {
        let text = "{% set total = 5 %}\nHello world.";
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: text.to_string(),
                origin_span: None,
            },
            kind: Kind::Jinja {
                variables: vec!["total".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(!diags.is_empty());

        if let Some(fix) = rule.autofix(&diags[0], &doc) {
            assert!(fix.safe);
            assert!(!fix.edits.is_empty());
            assert_eq!(fix.edits[0].replacement, "");
        }
    }

    #[test]
    fn non_jinja_template_ignored() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "{{ var }} in mustache".to_string(),
                origin_span: None,
            },
            kind: Kind::Mustache {
                variables: vec!["var".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "mustache templates not checked in v0");
    }

    #[test]
    fn multiple_unused_sets() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "{% set x = 1 %}\n{% set y = 2 %}\nDone.".to_string(),
                origin_span: None,
            },
            kind: Kind::Jinja {
                variables: vec!["x".to_string(), "y".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(diags.len() >= 2);
    }

    #[test]
    fn variable_used_in_block() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "{% set total = 5 %}\n{% if total > 0 %}Yes{% endif %}".to_string(),
                origin_span: None,
            },
            kind: Kind::Jinja {
                variables: vec!["total".to_string()],
            },
        };

        let rule = UnusedTemplateVariable;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "variable used in if block");
    }
}
