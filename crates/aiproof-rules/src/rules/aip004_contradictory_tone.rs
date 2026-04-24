//! AIP004: contradictory-tone
//!
//! Detect contradictory tone instructions in the prompt.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use regex::Regex;

const TONE_PAIRS: &[(&str, &str)] = &[
    ("concise", "detailed"),
    ("concise", "thorough"),
    ("concise", "comprehensive"),
    ("concise", "verbose"),
    ("brief", "thorough"),
    ("brief", "detailed"),
    ("short", "extensive"),
    ("casual", "formal"),
    ("informal", "formal"),
    ("friendly", "stern"),
    ("friendly", "strict"),
    ("playful", "serious"),
    ("funny", "serious"),
    ("terse", "verbose"),
];

pub struct ContradictoryTone;

impl Rule for ContradictoryTone {
    fn code(&self) -> &'static str {
        "AIP004"
    }

    fn name(&self) -> &'static str {
        "contradictory-tone"
    }

    fn category(&self) -> Category {
        Category::Clarity
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;
        let text_lower = text.to_lowercase();
        let mut diags = Vec::new();

        for &(a, b) in TONE_PAIRS {
            let regex_a = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(a))).unwrap();
            let regex_b = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(b))).unwrap();

            if let (Some(_), Some(m_b)) = (regex_a.find(&text_lower), regex_b.find(&text_lower)) {
                let span = project_span(doc, m_b.start()..m_b.end());
                diags.push(Diagnostic {
                    code: "AIP004".to_string(),
                    message: format!("contradictory tone: \"{}\" vs \"{}\"", a, b),
                    severity: Severity::Warning,
                    category: Category::Clarity,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP004".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(ContradictoryTone));
}
