//! AIP018: temperature-determinism-mismatch
//!
//! Detect prompts requesting deterministic output but with high temperature settings.
//! This applies to extracted SDK calls with temperature metadata.

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::{Document, Kind},
    rule::{Ctx, Rule},
    severity::Severity,
};

pub struct TemperatureDeterminismMismatch;

impl Rule for TemperatureDeterminismMismatch {
    fn code(&self) -> &'static str {
        "AIP018"
    }

    fn name(&self) -> &'static str {
        "temperature-determinism-mismatch"
    }

    fn category(&self) -> Category {
        Category::Portability
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        // Only check extracted Python/TypeScript with temperature metadata
        let temperature = match &doc.kind {
            Kind::ExtractedPython { temperature, .. } => *temperature,
            Kind::ExtractedTypeScript { temperature, .. } => *temperature,
            _ => return vec![],
        };

        let Some(temp) = temperature else {
            return vec![];
        };

        // Only flag if temperature allows randomness (> 0.3)
        if temp <= 0.3 {
            return vec![];
        }

        let text_lower = doc.prompt.text.to_lowercase();

        // Check for determinism cues
        let has_determinism_cue = [
            "always return exactly",
            "must produce the same",
            "deterministic",
            "reproducible",
            "consistent output",
            "verbatim",
            "identical",
        ]
        .iter()
        .any(|cue| text_lower.contains(cue));

        let mut diags = Vec::new();

        if has_determinism_cue {
            let span = doc.prompt.origin_span.clone().unwrap_or_else(|| {
                aiproof_core::span::Span::from_byte_range(&doc.source, 0..doc.prompt.text.len())
            });

            diags.push(Diagnostic {
                code: "AIP018".to_string(),
                message: format!(
                    "prompt asks for deterministic output but temperature={} allows randomness",
                    temp
                ),
                severity: Severity::Warning,
                category: Category::Portability,
                primary: span,
                labels: vec![],
                explain_url: Some("https://aiproof.dev/rules/AIP018".to_string()),
                fix: None,
            });
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(TemperatureDeterminismMismatch));
}
