//! AIP011: excessive-tokens
//!
//! Detect prompts that exceed a configurable token budget, helping stay within
//! context window or cost constraints.

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
    span::Span,
};

/// Default maximum tokens allowed in a prompt.
/// USER-WRITE: Kristian will tune this based on his prompt norms.
/// Typical values: 2000 (conservative), 4000 (default), 8000 (generous), 10000+ (research).
pub const DEFAULT_MAX_TOKENS: usize = 4000;

pub struct ExcessiveTokens;

impl Rule for ExcessiveTokens {
    fn code(&self) -> &'static str {
        "AIP011"
    }

    fn name(&self) -> &'static str {
        "excessive-tokens"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;

        // Heuristic: ~1.33 tokens per word (4 tokens / 3 words).
        let approx_tokens = (text.split_whitespace().count() * 4) / 3;

        let budget = ctx.max_tokens_budget.unwrap_or(DEFAULT_MAX_TOKENS);

        let mut diags = Vec::new();

        if approx_tokens > budget {
            let span = if let Some(origin_span) = &doc.prompt.origin_span {
                origin_span.clone()
            } else {
                // Fallback to start of document if no origin span.
                Span::from_byte_range(text, 0..0)
            };

            diags.push(Diagnostic {
                code: "AIP011".to_string(),
                message: format!(
                    "prompt exceeds token budget ({} > {})",
                    approx_tokens, budget
                ),
                severity: Severity::Warning,
                category: Category::Efficiency,
                primary: span,
                labels: vec![],
                explain_url: Some("https://aiproof.dev/rules/AIP011".to_string()),
                fix: None,
            });
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(ExcessiveTokens));
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::document::{Kind, PromptText, Role};
    use std::path::PathBuf;

    #[test]
    fn short_prompt_under_budget() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "You are helpful.".to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = ExcessiveTokens;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn long_prompt_exceeds_default_budget() {
        let long_text = "word ".repeat(5000); // ~6666 tokens
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: long_text,
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = ExcessiveTokens;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("exceeds token budget"));
    }

    #[test]
    fn long_prompt_within_custom_budget() {
        let long_text = "word ".repeat(5000); // ~6666 tokens
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: long_text,
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: Some(100_000),
        };

        let rule = ExcessiveTokens;
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "should not exceed custom budget");
    }

    #[test]
    fn boundary_at_budget_limit() {
        // Generate text that is just under 4000 tokens (~3000 words).
        let text = "word ".repeat(3000);
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text,
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = ExcessiveTokens;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0, "should be within budget");
    }

    #[test]
    fn boundary_just_over_budget_limit() {
        // Generate text that exceeds 4000 tokens (~3100 words).
        let text = "word ".repeat(3100);
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text,
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = ExcessiveTokens;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 1, "should exceed budget");
    }
}
