mod helpers;

use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::rule::{Ctx, Rule};
use aiproof_rules::rules::aip017_system_message_mismatch::SystemMessageMismatch;
use std::path::PathBuf;

fn make_system_doc(text: &str, role: Role) -> Document {
    Document {
        path: PathBuf::from("test.md"),
        role,
        source: text.to_string(),
        prompt: PromptText {
            text: text.to_string(),
            origin_span: None,
        },
        kind: Kind::Markdown,
    }
}

#[test]
fn flags_you_are_prompt_with_gemini_target() {
    let src = "You are a helpful assistant that answers questions.";
    let models = vec!["gemini-1.5-pro".to_string()];
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageMismatch;
    let ctx = Ctx {
        target_models: &models,
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(!diags.is_empty());
}

#[test]
fn clean_when_no_target_models() {
    let src = "You are a helpful assistant.";
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageMismatch;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn clean_when_not_starting_with_you_are() {
    let src = "Your task is to answer questions accurately.";
    let models = vec!["gemini-1.5-pro".to_string()];
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageMismatch;
    let ctx = Ctx {
        target_models: &models,
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn clean_when_user_role_with_you_are() {
    let src = "You are a helpful assistant.";
    let models = vec!["gemini-1.5-pro".to_string()];
    let doc = make_system_doc(src, Role::User);
    let rule = SystemMessageMismatch;
    let ctx = Ctx {
        target_models: &models,
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn case_insensitive_you_are_matching() {
    let src = "YOU ARE a helpful assistant.";
    let models = vec!["gemini-1.5-pro".to_string()];
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageMismatch;
    let ctx = Ctx {
        target_models: &models,
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(!diags.is_empty());
}
