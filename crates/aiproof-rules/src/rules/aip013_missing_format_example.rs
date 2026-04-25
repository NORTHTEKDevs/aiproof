//! AIP013: missing-format-example
//!
//! Detect imperative requests for structured output formats without an example or schema.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

// Imperative format triggers (case-insensitive)
static FORMAT_TRIGGERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(?:return|output|respond\s+with|produce|format|send|deliver|give|output)\s+(?:in\s+)?(?:json|xml|yaml|csv|structured\s+(?:output|format))",
    )
    .unwrap()
});

// Additional trigger: "structured output" or "structured format" without the verbs
static STRUCTURED_TRIGGER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bstructured\s+(?:output|format)\b").unwrap());

pub struct MissingFormatExample;

impl Rule for MissingFormatExample {
    fn code(&self) -> &'static str {
        "AIP013"
    }

    fn name(&self) -> &'static str {
        "missing-format-example"
    }

    fn category(&self) -> Category {
        Category::Behavior
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;
        let text_lower = text.to_lowercase();

        let mut diags = Vec::new();
        let mut matched_ranges = Vec::new();

        // Check primary format triggers
        for m in FORMAT_TRIGGERS.find_iter(text) {
            matched_ranges.push(m.range());
        }

        // Check structured format triggers
        for m in STRUCTURED_TRIGGER.find_iter(text) {
            // Only add if not already covered by the primary trigger
            if !matched_ranges.iter().any(|r| r.contains(&m.start())) {
                matched_ranges.push(m.range());
            }
        }

        // For each matched trigger, check for example signals
        for range in matched_ranges {
            let has_example = text_lower.contains("example:");
            let has_for_example = text_lower.contains("for example");
            let has_code_fence = text.contains("```");
            let has_json_block = text.contains('{') && text.contains('}');
            let has_xml = text.contains('<') && text.contains('>');

            if !has_example && !has_for_example && !has_code_fence && !has_json_block && !has_xml {
                let span = project_span(doc, range.clone());
                diags.push(Diagnostic {
                    code: "AIP013".to_string(),
                    message: "structured output requested without an example or schema".to_string(),
                    severity: Severity::Info,
                    category: Category::Behavior,
                    primary: span,
                    labels: vec![],
                    explain_url: Some(
                        "https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP013.md"
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
    out.push(Box::new(MissingFormatExample));
}
