//! AIP008: known-jailbreak-pattern
//!
//! Detect known jailbreak and prompt injection signatures in prompt text.
//! This list is intentionally curated for high-confidence detections.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};

pub(crate) const SIGNATURES: &[&str] = &[
    "ignore previous instructions",
    "ignore all previous",
    "disregard all prior",
    "you are now dan",
    "do anything now",
    // USER-WRITE: extend this list with 10-15 additional jailbreak signatures
    // seen in the wild or red-team literature. Each entry is a lowercase substring
    // that will match case-insensitively against doc.prompt.text.
];

pub struct KnownJailbreakPattern;

impl Rule for KnownJailbreakPattern {
    fn code(&self) -> &'static str {
        "AIP008"
    }

    fn name(&self) -> &'static str {
        "known-jailbreak-pattern"
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text_lower = doc.prompt.text.to_lowercase();
        let mut diags = Vec::new();

        for &signature in SIGNATURES {
            if let Some(pos) = text_lower.find(signature) {
                let span = project_span(doc, pos..pos + signature.len());
                diags.push(Diagnostic {
                    code: "AIP008".to_string(),
                    message: format!("known jailbreak/injection pattern: \"{}\"", signature),
                    severity: Severity::Error,
                    category: Category::Security,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP008".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }
}

pub fn signature_count() -> usize {
    SIGNATURES.len()
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(KnownJailbreakPattern));
}
