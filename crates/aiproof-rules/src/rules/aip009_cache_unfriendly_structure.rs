//! AIP009: cache-unfriendly-structure
//!
//! Detect variable interpolation within the first ~1024-token prefix of system prompts.
//! Anthropic prompt caching requires a stable prefix to produce cache hits; inserting
//! variables too early defeats caching.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::{Document, Role},
    rule::{Ctx, Rule},
    severity::Severity,
};
use regex::Regex;

/// Threshold token count for the prefix. Variables before this point defeat caching.
/// USER-WRITE: Kristian will tune this based on his prompt corpus norms.
/// Typical values: 1024 (Opus default), 2048 (Haiku environments), 512 (short prompts).
pub const PREFIX_TOKEN_THRESHOLD: usize = 1024;

pub struct CacheUnfriendlyStructure;

impl Rule for CacheUnfriendlyStructure {
    fn code(&self) -> &'static str {
        "AIP009"
    }

    fn name(&self) -> &'static str {
        "cache-unfriendly-structure"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        // Only applies to system-role prompts.
        if doc.role != Role::System {
            return diags;
        }

        let text = &doc.prompt.text;

        // Regex to match interpolation sites: {var}, {{ var }}, {0}, {1}, etc.
        // Matches: { optional whitespace, identifier or digit(s), optional whitespace, } or
        // {{ optional whitespace, identifier or digit(s), optional whitespace, }}
        let interp_regex = Regex::new(r"\{\{?\s*(?:[A-Za-z_][\w.]*|[0-9]+)\s*\}\}?")
            .expect("regex should compile");

        for m in interp_regex.find_iter(text) {
            let prefix_text = &text[..m.start()];
            // Heuristic: ~1.33 tokens per word (4 tokens / 3 words).
            let approx_tokens = (prefix_text.split_whitespace().count() * 4) / 3;

            if approx_tokens < PREFIX_TOKEN_THRESHOLD {
                let span = project_span(doc, m.start()..m.end());
                diags.push(Diagnostic {
                    code: "AIP009".to_string(),
                    message: format!(
                        "variable content at token ~{} defeats prompt caching (threshold: ~{})",
                        approx_tokens, PREFIX_TOKEN_THRESHOLD
                    ),
                    severity: Severity::Warning,
                    category: Category::Efficiency,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP009".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(CacheUnfriendlyStructure));
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::document::PromptText;
    use std::path::PathBuf;

    #[test]
    fn short_system_prompt_with_early_variable() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "You are helpful. {var} Please respond.".to_string(),
                origin_span: None,
            },
            kind: aiproof_core::document::Kind::PlainText,
        };

        let rule = CacheUnfriendlyStructure;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("variable content at token"));
    }

    #[test]
    fn long_system_prompt_with_late_variable() {
        let long_prefix = "You are a helpful assistant. ".to_string() + &"word ".repeat(900); // ~1350+ tokens of prefix (900 words * 4/3)
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: long_prefix + "{var}",
                origin_span: None,
            },
            kind: aiproof_core::document::Kind::PlainText,
        };

        let rule = CacheUnfriendlyStructure;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "late variable should not trigger");
    }

    #[test]
    fn user_role_ignores_variables() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::User,
            source: String::new(),
            prompt: PromptText {
                text: "Please help with {var}".to_string(),
                origin_span: None,
            },
            kind: aiproof_core::document::Kind::PlainText,
        };

        let rule = CacheUnfriendlyStructure;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "user role should never trigger");
    }

    #[test]
    fn multiple_early_variables() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "You are {x}. {y} Please respond.".to_string(),
                origin_span: None,
            },
            kind: aiproof_core::document::Kind::PlainText,
        };

        let rule = CacheUnfriendlyStructure;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(diags.len() >= 2, "both early variables should trigger");
    }
}
