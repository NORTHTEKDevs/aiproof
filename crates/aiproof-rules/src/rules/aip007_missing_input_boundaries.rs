//! AIP007: missing-input-boundaries
//!
//! Detect single-brace Python-style interpolations in system prompts
//! without XML delimiter wrapping. Provides autofix to wrap in tags.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic, Edit, Fix},
    document::{Document, Role},
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

static INTERPOLATION_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\w+\}").unwrap());

pub struct MissingInputBoundaries;

impl Rule for MissingInputBoundaries {
    fn code(&self) -> &'static str {
        "AIP007"
    }

    fn name(&self) -> &'static str {
        "missing-input-boundaries"
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        // Only check system prompts
        if doc.role != Role::System {
            return vec![];
        }

        let text = &doc.prompt.text;
        let mut diags = Vec::new();

        for m in INTERPOLATION_PATTERN.find_iter(text) {
            let match_pos = m.start();

            // Look at preceding 30 chars for XML delimiter or backtick
            let check_start = match_pos.saturating_sub(30);
            let preceding = &text[check_start..match_pos];

            let has_xml_or_backtick = preceding.contains('<') || preceding.contains('`');

            if !has_xml_or_backtick {
                let span = project_span(doc, m.start()..m.end());
                diags.push(Diagnostic {
                    code: "AIP007".to_string(),
                    message: "single-brace interpolation not wrapped in XML tags — add <user_input>…</user_input>"
                        .to_string(),
                    severity: Severity::Warning,
                    category: Category::Security,
                    primary: span.clone(),
                    labels: vec![],
                    explain_url: Some("https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP007.md".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }

    fn autofix(&self, diag: &Diagnostic, doc: &Document) -> Option<Fix> {
        let text = &doc.prompt.text;

        // Extract the interpolation from the span (in source coords)
        // We need to get the byte range relative to prompt.text
        let base = doc
            .prompt
            .origin_span
            .as_ref()
            .map(|s| s.byte_range.start)
            .unwrap_or(0);

        let local_start = diag.primary.byte_range.start.saturating_sub(base);
        let local_end = diag.primary.byte_range.end.saturating_sub(base);

        if local_start >= text.len() || local_end > text.len() || local_start >= local_end {
            return None;
        }

        let matched = &text[local_start..local_end];

        Some(Fix {
            description: "wrap interpolation in <user_input> tags".into(),
            edits: vec![Edit {
                span: diag.primary.clone(),
                replacement: format!("<user_input>{}</user_input>", matched),
            }],
            safe: true,
        })
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(MissingInputBoundaries));
}
