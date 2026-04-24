//! AIP005: unescaped-user-input
//!
//! Detect user input interpolations in system prompts that aren't wrapped
//! in XML delimiter tags or other boundary markers.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::{Document, Role},
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

static INTERPOLATION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{?\s*(\w+)\s*\}?\}").unwrap());

pub struct UnescapedUserInput;

impl Rule for UnescapedUserInput {
    fn code(&self) -> &'static str {
        "AIP005"
    }

    fn name(&self) -> &'static str {
        "unescaped-user-input"
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

            // Look at 30 chars preceding the match for boundary delimiters
            let check_start = match_pos.saturating_sub(30);
            let preceding = &text[check_start..match_pos];

            // Check for boundary delimiters: <, >, backtick, triple quotes, [
            let has_boundary = preceding.contains('<')
                || preceding.contains('>')
                || preceding.contains('`')
                || preceding.contains("\"\"\"")
                || preceding.contains("'''")
                || preceding.contains('[');

            if !has_boundary {
                let span = project_span(doc, m.start()..m.end());
                diags.push(Diagnostic {
                    code: "AIP005".to_string(),
                    message: "user interpolation not wrapped in delimiter tags — vulnerable to prompt injection".to_string(),
                    severity: Severity::Warning,
                    category: Category::Security,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP005".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(UnescapedUserInput));
}
