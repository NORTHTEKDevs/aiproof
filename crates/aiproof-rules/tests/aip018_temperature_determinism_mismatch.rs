mod helpers;

use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::rule::{Ctx, Rule};
use aiproof_core::span::Span;
use aiproof_rules::rules::aip018_temperature_determinism_mismatch::TemperatureDeterminismMismatch;
use std::path::PathBuf;

#[test]
fn flags_high_temp_with_deterministic_prompt() {
    let src = "Always return exactly this JSON schema.";
    let doc = Document {
        path: PathBuf::from("test.py"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::ExtractedPython {
            call_site: Span::from_byte_range(src, 0..10),
            temperature: Some(0.7),
        },
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(!diags.is_empty(), "expected at least one warning");
    assert_eq!(diags[0].code, "AIP018");
}

#[test]
fn clean_when_temperature_is_zero() {
    let src = "Always return exactly this JSON schema.";
    let doc = Document {
        path: PathBuf::from("test.py"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::ExtractedPython {
            call_site: Span::from_byte_range(src, 0..10),
            temperature: Some(0.0),
        },
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 0);
}

#[test]
fn clean_when_no_determinism_cue() {
    let src = "Please respond to the user's question.";
    let doc = Document {
        path: PathBuf::from("test.py"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::ExtractedPython {
            call_site: Span::from_byte_range(src, 0..10),
            temperature: Some(0.7),
        },
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 0);
}

#[test]
fn clean_for_non_extracted_docs() {
    let src = "Always return exactly this JSON schema.";
    let doc = Document {
        path: PathBuf::from("test.md"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::Markdown,
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 0);
}

#[test]
fn flags_multiple_determinism_cues() {
    let src = "Must produce the same deterministic output every time.";
    let doc = Document {
        path: PathBuf::from("test.py"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::ExtractedTypeScript {
            call_site: Span::from_byte_range(src, 0..10),
            temperature: Some(0.5),
        },
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(!diags.is_empty());
}

#[test]
fn clean_when_temperature_at_boundary() {
    let src = "Always return exactly this JSON schema.";
    let doc = Document {
        path: PathBuf::from("test.py"),
        role: Role::System,
        source: src.to_string(),
        prompt: PromptText {
            text: src.to_string(),
            origin_span: None,
        },
        kind: Kind::ExtractedPython {
            call_site: Span::from_byte_range(src, 0..10),
            temperature: Some(0.3),
        },
    };

    let rule = TemperatureDeterminismMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 0);
}
