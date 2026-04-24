//! AIP002: ambiguous-output-format
//!
//! Detect requests for structured output formats without a schema or example.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

// Regex for whole-word format names (case-insensitive)
static JSON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bjson\b").unwrap());
static XML_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bxml\b").unwrap());
static YAML_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\byaml\b").unwrap());
static CSV_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bcsv\b").unwrap());

pub struct AmbiguousOutputFormat;

impl Rule for AmbiguousOutputFormat {
    fn code(&self) -> &'static str {
        "AIP002"
    }

    fn name(&self) -> &'static str {
        "ambiguous-output-format"
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

        // Check if any format is mentioned
        let regexes = [&JSON_RE, &XML_RE, &YAML_RE, &CSV_RE];
        let mut diags = Vec::new();

        for regex in &regexes {
            if let Some(m) = regex.find(text) {
                // Check for schema/example signals
                let has_json_block = text.contains('{') && text.contains('}');
                let has_xml = text.contains('<') && text.contains('>');
                let has_example = text_lower.contains("example:");
                let has_schema = text_lower.contains("schema:");
                let has_code_fence = text.contains("```");

                if !has_json_block && !has_xml && !has_example && !has_schema && !has_code_fence {
                    let format_name = &text[m.start()..m.end()];
                    let span = project_span(doc, m.start()..m.end());
                    diags.push(Diagnostic {
                        code: "AIP002".to_string(),
                        message: format!(
                            "prompt requests {} output without a schema or example",
                            format_name
                        ),
                        severity: Severity::Warning,
                        category: Category::Clarity,
                        primary: span,
                        labels: vec![],
                        explain_url: Some("https://aiproof.dev/rules/AIP002".to_string()),
                        fix: None,
                    });
                }
            }
        }

        diags
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(AmbiguousOutputFormat));
}
