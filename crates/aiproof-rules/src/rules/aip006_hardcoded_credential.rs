//! AIP006: hardcoded-credential
//!
//! Detect hardcoded API keys and credentials in prompt text.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic, Edit, Fix},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};
use once_cell::sync::Lazy;
use regex::Regex;

static PATTERNS: Lazy<Vec<(&'static str, Regex)>> = Lazy::new(|| {
    vec![
        (
            "anthropic",
            Regex::new(r"sk-ant-api03-[A-Za-z0-9_-]{20,}").unwrap(),
        ),
        ("openai", Regex::new(r"\bsk-[A-Za-z0-9]{20,}\b").unwrap()),
        ("aws-access", Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap()),
        (
            "github-pat",
            Regex::new(r"\bghp_[A-Za-z0-9]{32,}\b").unwrap(),
        ),
        ("slack-bot", Regex::new(r"\bxoxb-[0-9A-Za-z-]+\b").unwrap()),
        (
            "google-api",
            Regex::new(r"\bAIza[0-9A-Za-z_-]{34,}\b").unwrap(),
        ),
        (
            "stripe-live",
            Regex::new(r"\bsk_live_[A-Za-z0-9]{24,}\b").unwrap(),
        ),
    ]
});

pub struct HardcodedCredential;

impl Rule for HardcodedCredential {
    fn code(&self) -> &'static str {
        "AIP006"
    }

    fn name(&self) -> &'static str {
        "hardcoded-credential"
    }

    fn category(&self) -> Category {
        Category::Security
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;
        let mut diags = Vec::new();

        for (provider, regex) in PATTERNS.iter() {
            for m in regex.find_iter(text) {
                let span = project_span(doc, m.start()..m.end());
                diags.push(Diagnostic {
                    code: "AIP006".to_string(),
                    message: format!("hardcoded {} credential in prompt text", provider),
                    severity: Severity::Error,
                    category: Category::Security,
                    primary: span,
                    labels: vec![],
                    explain_url: Some("https://aiproof.dev/rules/AIP006".to_string()),
                    fix: None,
                });
            }
        }

        diags
    }

    fn autofix(&self, diag: &Diagnostic, _doc: &Document) -> Option<Fix> {
        Some(Fix {
            description: "redact the matched credential".into(),
            edits: vec![Edit {
                span: diag.primary.clone(),
                replacement: "***REDACTED***".into(),
            }],
            safe: true,
        })
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(HardcodedCredential));
}
