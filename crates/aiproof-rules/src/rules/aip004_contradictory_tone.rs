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
use once_cell::sync::Lazy;
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

// Compile tone-pair regexes once. Inputs are lowercase ASCII words, so simple
// word-boundary patterns are sufficient and cheaper than per-call Regex::new.
static TONE_REGEXES: Lazy<Vec<(&'static str, &'static str, Regex, Regex)>> = Lazy::new(|| {
    TONE_PAIRS
        .iter()
        .map(|&(a, b)| {
            let ra = Regex::new(&format!(r"\b{}\b", regex::escape(a))).unwrap();
            let rb = Regex::new(&format!(r"\b{}\b", regex::escape(b))).unwrap();
            (a, b, ra, rb)
        })
        .collect()
});

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

        for (a, b, regex_a, regex_b) in TONE_REGEXES.iter() {
            if let (Some(_), Some(m_b)) = (regex_a.find(&text_lower), regex_b.find(&text_lower)) {
                let span = project_span(doc, m_b.start()..m_b.end());
                diags.push(Diagnostic {
                    code: "AIP004".to_string(),
                    message: format!("contradictory tone: \"{}\" vs \"{}\"", a, b),
                    severity: Severity::Warning,
                    category: Category::Clarity,
                    primary: span,
                    labels: vec![],
                    explain_url: Some(
                        "https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP004.md"
                            .to_string(),
                    ),
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
