//! AIP017: system-message-mismatch
//!
//! Detect Anthropic-style system prompts ("You are...") when targeting Gemini models.
//! Gemini handles system messages with different semantics.

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::{Document, Role},
    rule::{Ctx, Rule},
    severity::Severity,
};

pub struct SystemMessageMismatch;

impl Rule for SystemMessageMismatch {
    fn code(&self) -> &'static str {
        "AIP017"
    }

    fn name(&self) -> &'static str {
        "system-message-mismatch"
    }

    fn category(&self) -> Category {
        Category::Portability
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic> {
        // Only apply when target models include Gemini
        let targets_gemini = ctx
            .target_models
            .iter()
            .any(|m| m.to_lowercase().starts_with("gemini"));

        if !targets_gemini {
            return vec![];
        }

        // Only check system role prompts
        if doc.role != Role::System {
            return vec![];
        }

        let first_line = doc.prompt.text.lines().next().unwrap_or("").trim();
        let matches_anthropic_shape = first_line.to_lowercase().starts_with("you are ");

        let mut diags = Vec::new();

        if matches_anthropic_shape {
            let span =
                doc.prompt.origin_span.clone().unwrap_or_else(|| {
                    aiproof_core::span::Span::from_byte_range(&doc.source, 0..0)
                });

            diags.push(Diagnostic {
                code: "AIP017".to_string(),
                message:
                    "system prompt opens with Anthropic-style 'You are...' — Gemini handles system messages differently; consider restructuring"
                        .to_string(),
                severity: Severity::Info,
                category: Category::Portability,
                primary: span,
                labels: vec![],
                explain_url: Some("https://aiproof.dev/rules/AIP017".to_string()),
                fix: None,
            });
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(SystemMessageMismatch));
}
