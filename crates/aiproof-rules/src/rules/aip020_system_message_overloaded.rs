//! AIP020: system-message-overloaded
//!
//! Flags system prompts that try to do too much in one message.
//! Applies two independent gates: size (>1500 tokens) and imperative density (>8 imperatives).

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::{Document, Role},
    rule::{Ctx, Rule},
    severity::Severity,
};

pub struct SystemMessageOverloaded;

impl Rule for SystemMessageOverloaded {
    fn code(&self) -> &'static str {
        "AIP020"
    }

    fn name(&self) -> &'static str {
        "system-message-overloaded"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        // Only apply to system role prompts
        if doc.role != Role::System {
            return vec![];
        }

        let text_lower = doc.prompt.text.to_lowercase();

        // Gate A: approximate token count
        let approx_tokens = doc.prompt.text.split_whitespace().count() * 4 / 3;
        let gate_a_fires = approx_tokens > 1500;

        // Gate B: imperative clauses
        let imperatives = [
            "you must",
            "you should",
            "always ",
            "never ",
            "do not ",
            "don't ",
            "please ",
            "ensure ",
            "make sure ",
            "remember to",
            "be sure to",
        ];

        let imperative_count: usize = imperatives
            .iter()
            .map(|imp| text_lower.matches(imp).count())
            .sum();

        let gate_b_fires = imperative_count > 8;

        // If neither gate fires, emit nothing
        if !gate_a_fires && !gate_b_fires {
            return vec![];
        }

        // Build reason message
        let mut reason_parts = Vec::new();
        if gate_a_fires {
            reason_parts.push(format!("approx {} tokens (>1500)", approx_tokens));
        }
        if gate_b_fires {
            reason_parts.push(format!("{} imperative clauses (>8)", imperative_count));
        }

        let reason = reason_parts.join("; ");

        let span = doc
            .prompt
            .origin_span
            .clone()
            .unwrap_or_else(|| aiproof_core::span::Span::from_byte_range(&doc.source, 0..0));

        vec![Diagnostic {
            code: "AIP020".to_string(),
            message: format!("system message overloaded: {}", reason),
            severity: Severity::Info,
            category: Category::Efficiency,
            primary: span,
            labels: vec![],
            explain_url: Some("https://aiproof.dev/rules/AIP020".to_string()),
            fix: None,
        }]
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(SystemMessageOverloaded));
}
