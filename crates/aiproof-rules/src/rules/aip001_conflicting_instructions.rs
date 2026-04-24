//! AIP001: conflicting-instructions
//!
//! Detect explicit contradictions by matching pairs of known-conflicting imperatives.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};

const PAIRS: &[(&str, &str)] = &[
    ("only output json", "explain your reasoning"),
    ("only output json", "add commentary"),
    ("respond in json", "use natural language"),
    ("do not explain", "explain your reasoning"),
    ("do not apologize", "apologize if"),
    ("never refuse", "refuse if"),
    ("be creative", "stick to the facts"),
    ("be conservative", "be bold"),
];

pub struct ConflictingInstructions;

impl Rule for ConflictingInstructions {
    fn code(&self) -> &'static str {
        "AIP001"
    }

    fn name(&self) -> &'static str {
        "conflicting-instructions"
    }

    fn category(&self) -> Category {
        Category::Clarity
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text_lower = doc.prompt.text.to_lowercase();
        let mut diags = Vec::new();

        for &(a, b) in PAIRS {
            let has_a = text_lower.contains(a);
            let has_b = text_lower.contains(b);

            if has_a && has_b {
                // Find position of second string for the diagnostic
                if let Some(pos) = text_lower.find(b) {
                    let span = project_span(doc, pos..pos + b.len());
                    diags.push(Diagnostic {
                        code: "AIP001".to_string(),
                        message: format!(
                            "conflicting instruction: \"{}\" contradicts \"{}\" above",
                            b, a
                        ),
                        severity: Severity::Warning,
                        category: Category::Clarity,
                        primary: span,
                        labels: vec![],
                        explain_url: Some("https://aiproof.dev/rules/AIP001".to_string()),
                        fix: None,
                    });
                }
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(ConflictingInstructions));
}
