//! AIP015: unhandled-placeholder
//!
//! Detect unresolved placeholders in the prompt (TODO, FIXME, XXX, PLACEHOLDER, ...TBD).

use crate::util::{is_prompt_shaped, project_span};
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

// Match TODO, FIXME, XXX (uppercase), PLACEHOLDER, and ...TBD (case-insensitive for TBD)
// Note: \bTODO\b matches "TODO" and "TODOs" because the second \b is at word boundary after O,
// which also matches the boundary before 's'. This is acceptable -- all forms are unresolved.
static PLACEHOLDER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(TODO|FIXME|XXX|PLACEHOLDER)\b|\.\.\.TBD\b").unwrap());

pub struct UnhandledPlaceholder;

impl Rule for UnhandledPlaceholder {
    fn code(&self) -> &'static str {
        "AIP015"
    }

    fn name(&self) -> &'static str {
        "unhandled-placeholder"
    }

    fn category(&self) -> Category {
        Category::Behavior
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        if !is_prompt_shaped(doc) {
            return Vec::new();
        }
        let text = &doc.prompt.text;
        let mut diags = Vec::new();

        for m in PLACEHOLDER_RE.find_iter(text) {
            let matched_text = &text[m.start()..m.end()];
            let span = project_span(doc, m.range());
            diags.push(Diagnostic {
                code: "AIP015".to_string(),
                message: format!("unresolved placeholder \"{}\" in prompt body", matched_text),
                severity: Severity::Warning,
                category: Category::Behavior,
                primary: span,
                labels: vec![],
                explain_url: Some("https://aiproof.dev/rules/AIP015".to_string()),
                fix: None,
            });
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(UnhandledPlaceholder));
}
