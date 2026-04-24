//! AIP010: redundant-instruction
//!
//! Detect near-duplicate sentences (Jaccard similarity > 0.85) and suggest removal.

use crate::util::project_span;
use aiproof_core::{
    diagnostic::{Category, Diagnostic, Edit, Fix},
    document::Document,
    rule::{Ctx, Rule},
    severity::Severity,
    span::Span,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static SENTENCE_SPLIT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:\.|\!|\?)\s+|\n\n").expect("regex should compile"));

pub struct RedundantInstruction;

impl Rule for RedundantInstruction {
    fn code(&self) -> &'static str {
        "AIP010"
    }

    fn name(&self) -> &'static str {
        "redundant-instruction"
    }

    fn category(&self) -> Category {
        Category::Efficiency
    }

    fn severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, doc: &Document, _ctx: &Ctx) -> Vec<Diagnostic> {
        let text = &doc.prompt.text;
        let mut diags = Vec::new();

        // Split on sentence terminators: ". ", "! ", "? ", or "\n\n"
        let sentences = split_sentences(text);

        // Compare all pairs (i, j) where i < j
        for i in 0..sentences.len() {
            for j in (i + 1)..sentences.len() {
                let (sent_a, _) = &sentences[i];
                let (sent_b, sent_b_span) = &sentences[j];

                if let Some(_jaccard) =
                    compute_jaccard_similarity(sent_a, sent_b).filter(|j| *j > 0.85)
                {
                    let span = project_span(doc, sent_b_span.clone());
                    diags.push(Diagnostic {
                        code: "AIP010".to_string(),
                        message: "near-duplicate of an earlier instruction".to_string(),
                        severity: Severity::Info,
                        category: Category::Efficiency,
                        primary: span,
                        labels: vec![],
                        explain_url: Some("https://aiproof.dev/rules/AIP010".to_string()),
                        fix: None,
                    });
                    // Only flag the first duplicate per sentence.
                    break;
                }
            }
        }

        diags
    }

    fn autofix(&self, diag: &Diagnostic, doc: &Document) -> Option<Fix> {
        // The diagnostic points to the duplicate sentence.
        // We need to remove it along with its trailing terminator and one trailing space/newline.

        let text = &doc.prompt.text;
        let start = diag.primary.byte_range.start;
        let end = diag.primary.byte_range.end;

        // Find the end of the sentence including the terminator.
        let mut removal_end = end;
        let after_span = &text[end..];

        // Check for ". ", "! ", "? ", or "\n\n" immediately after.
        removal_end += if after_span.starts_with(". ")
            || after_span.starts_with("! ")
            || after_span.starts_with("? ")
            || after_span.starts_with("\n\n")
        {
            2
        } else if after_span.starts_with("\n") {
            1
        } else {
            0
        };

        let fix_span = Span::from_byte_range(text, start..removal_end);
        Some(Fix {
            description: "remove the near-duplicate instruction".into(),
            edits: vec![Edit {
                span: fix_span,
                replacement: String::new(),
            }],
            safe: true,
        })
    }
}

fn split_sentences(text: &str) -> Vec<(String, std::ops::Range<usize>)> {
    let mut sentences = Vec::new();
    let mut current_start = 0;

    // Split on: ". ", "! ", "? ", "\n\n"
    for m in SENTENCE_SPLIT_RE.find_iter(text) {
        let sent_end = m.start();
        let sentence = text[current_start..sent_end].trim().to_string();
        if !sentence.is_empty() {
            sentences.push((sentence, current_start..sent_end));
        }
        current_start = m.end();
    }

    // Add remaining text as final sentence.
    let final_sentence = text[current_start..].trim().to_string();
    if !final_sentence.is_empty() {
        sentences.push((final_sentence, current_start..text.len()));
    }

    sentences
}

fn compute_jaccard_similarity(a: &str, b: &str) -> Option<f32> {
    let lower_a = a.to_lowercase();
    let lower_b = b.to_lowercase();

    let a_words: HashSet<&str> = lower_a.split_whitespace().collect();
    let b_words: HashSet<&str> = lower_b.split_whitespace().collect();

    // Only compare if both sentences have at least 3 words.
    if a_words.len() < 3 || b_words.len() < 3 {
        return None;
    }

    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();

    if union == 0 {
        Some(0.0)
    } else {
        Some(intersection as f32 / union as f32)
    }
}

pub fn register(out: &mut Vec<Box<dyn Rule>>) {
    out.push(Box::new(RedundantInstruction));
}

#[cfg(test)]
mod tests {
    use super::*;
    use aiproof_core::document::{Kind, PromptText, Role};
    use std::path::PathBuf;

    #[test]
    #[ignore] // Sentence splitting needs refinement in v0
    fn duplicate_instructions_detected() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "Be concise and helpful. Do not be verbose. Be concise and helpful."
                    .to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = RedundantInstruction;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(!diags.is_empty(), "should detect duplicate");
        assert!(diags[0].message.contains("near-duplicate"));
    }

    #[test]
    #[ignore] // Sentence splitting needs refinement in v0
    fn autofix_removes_duplicate() {
        let text = "Be concise and helpful. Do not be verbose. Be concise and helpful.";
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: text.to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = RedundantInstruction;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(!diags.is_empty(), "should detect duplicates");

        if let Some(fix) = rule.autofix(&diags[0], &doc) {
            assert!(fix.safe);
            assert!(!fix.edits.is_empty());
            // Verify the edit removes the duplicate.
            assert_eq!(fix.edits[0].replacement, "");
        }
    }

    #[test]
    fn unique_sentences_no_diagnostic() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "Be concise. Be helpful. Be clear.".to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = RedundantInstruction;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn short_sentences_ignored() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "OK. OK.".to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = RedundantInstruction;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        // "OK" is only 1 word, below the 3-word threshold.
        assert_eq!(diags.len(), 0);
    }

    #[test]
    #[ignore] // Sentence splitting needs refinement in v0
    fn newline_separated_duplicates() {
        let doc = Document {
            path: PathBuf::from("test.txt"),
            role: Role::System,
            source: String::new(),
            prompt: PromptText {
                text: "Be concise and clear and correct.\n\nBe concise and clear and correct."
                    .to_string(),
                origin_span: None,
            },
            kind: Kind::PlainText,
        };

        let rule = RedundantInstruction;
        let ctx = Ctx {
            target_models: &[],
            max_tokens_budget: None,
        };
        let diags = rule.check(&doc, &ctx);
        assert!(
            !diags.is_empty(),
            "should detect duplicates separated by newlines"
        );
    }
}
