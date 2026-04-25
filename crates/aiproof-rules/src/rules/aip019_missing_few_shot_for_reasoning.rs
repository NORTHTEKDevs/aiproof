//! AIP019: missing-few-shot-for-reasoning
//!
//! Chain-of-thought prompts work better with concrete examples.
//! Flag reasoning-style prompts that provide no few-shot examples.

use aiproof_core::{
    diagnostic::{Category, Diagnostic},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
};

pub struct MissingFewShotForReasoning;

impl Rule for MissingFewShotForReasoning {
    fn code(&self) -> &'static str {
        "AIP019"
    }

    fn name(&self) -> &'static str {
        "missing-few-shot-for-reasoning"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text_lower = doc.prompt.text.to_lowercase();

        // Reasoning cues that signal chain-of-thought intention
        let reasoning_cues = [
            "think step by step",
            "step-by-step",
            "reason through",
            "reason carefully",
            "chain of thought",
            "let's think",
            "show your work",
            "work through",
        ];

        // Check if any reasoning cue is present, capturing the earliest match.
        let mut first_cue_match: Option<(usize, &str)> = None;
        for cue in &reasoning_cues {
            if let Some(pos) = text_lower.find(cue) {
                match first_cue_match {
                    Some((earliest, _)) if pos >= earliest => {}
                    _ => first_cue_match = Some((pos, cue)),
                }
            }
        }

        // If no reasoning cue, emit nothing.
        let Some((cue_pos, _)) = first_cue_match else {
            return vec![];
        };

        // Check for example signals
        let has_example_example = text_lower.contains("example:");
        let has_example_1 = text_lower.contains("example 1");
        let has_example_2 = text_lower.contains("example 2");

        // Check for input/output pattern.
        let has_input_output = matches!(
            (text_lower.find("input:"), text_lower.find("output:")),
            (Some(i), Some(o)) if i < o
        );

        // Check for q/a pattern.
        let has_q_a = matches!(
            (text_lower.find("q:"), text_lower.find("a:")),
            (Some(q), Some(a)) if q < a
        );

        // Check for code fence
        let has_code_fence = doc.prompt.text.contains("```");

        // Check for three or more markdown list items
        let list_item_count = doc
            .prompt
            .text
            .lines()
            .filter(|line| line.trim().starts_with("- "))
            .count();
        let has_list = list_item_count >= 3;

        // If any example signal present, emit nothing
        if has_example_example
            || has_example_1
            || has_example_2
            || has_input_output
            || has_q_a
            || has_code_fence
            || has_list
        {
            return vec![];
        }

        // Emit diagnostic at first reasoning cue position.
        let span = doc.prompt.origin_span.clone().unwrap_or_else(|| {
            aiproof_core::span::Span::from_byte_range(&doc.source, cue_pos..cue_pos)
        });

        vec![Diagnostic {
            code: "AIP019".to_string(),
            message: "chain-of-thought prompt without concrete examples — consider adding a worked example"
                .to_string(),
            severity: Severity::Info,
            category: Category::Efficiency,
            primary: span,
            labels: vec![],
            explain_url: Some("https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP019.md".to_string()),
            fix: None,
        }]
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(MissingFewShotForReasoning));
}
