//! AIP016: claude-specific-tags-on-gpt
//!
//! Detect Anthropic-specific XML tags when targeting GPT models.
//! These tags are ignored by GPT models and will not function as intended.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};

const ANTHROPIC_TAGS: &[&str] = &[
    "<thinking>",
    "</thinking>",
    "<scratchpad>",
    "</scratchpad>",
    "<reflection>",
    "</reflection>",
];

pub struct ClaudeSpecificTagsOnGpt;

impl Rule for ClaudeSpecificTagsOnGpt {
    fn code(&self) -> &'static str {
        "AIP016"
    }

    fn name(&self) -> &'static str {
        "claude-specific-tags-on-gpt"
    }

    fn category(&self) -> Category {
        Category::Portability
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic> {
        // Only apply when target models include GPT-like models
        let targets_gpt = ctx
            .target_models
            .iter()
            .any(|m| m.to_lowercase().starts_with("gpt-") || m.to_lowercase() == "openai");

        if !targets_gpt {
            return vec![];
        }

        let text_lower = doc.prompt.text.to_lowercase();
        let mut diags = Vec::new();

        for &tag in ANTHROPIC_TAGS {
            let tag_lower = tag.to_lowercase();
            if let Some(pos) = text_lower.find(&tag_lower) {
                let span = project_span(doc, pos..pos + tag.len());
                diags.push(Diagnostic {
                    code: "AIP016".to_string(),
                    message:
                        "<thinking>-style tags are Anthropic conventions — GPT models do not parse them specially"
                            .to_string(),
                    severity: Severity::Warning,
                    category: Category::Portability,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP016".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(ClaudeSpecificTagsOnGpt));
}
