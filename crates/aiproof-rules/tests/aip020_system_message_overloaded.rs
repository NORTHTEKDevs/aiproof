mod helpers;

use aiproof_core::document::{Document, Kind, PromptText, Role};
use aiproof_core::rule::{Ctx, Rule};
use aiproof_rules::rules::aip020_system_message_overloaded::SystemMessageOverloaded;
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
fn short_system_prompt_clean() {
    let src = "You are a helpful assistant.";
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn large_system_prompt_fires_gate_a() {
    // Create a prompt with ~3000 words (should exceed 1500 tokens)
    let long_text = "word ".repeat(3000);
    let doc = make_system_doc(&long_text, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP020");
    assert!(diags[0].message.contains("approx"));
    assert!(diags[0].message.contains("tokens"));
}

#[test]
fn user_role_long_prompt_clean() {
    let long_text = "word ".repeat(3000);
    let doc = make_system_doc(&long_text, Role::User);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn system_prompt_with_many_imperatives_fires_gate_b() {
    let src = "You must be helpful. You must be accurate. You should be concise. \
               You should verify facts. Always check your work. Never make assumptions. \
               Please be professional. Do not speculate. Make sure to cite sources. \
               Ensure accuracy. Remember to be clear.";
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP020");
    assert!(diags[0].message.contains("imperative"));
}

#[test]
fn few_imperatives_clean() {
    let src = "You must be helpful. You should be accurate. Always check your work.";
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert!(diags.is_empty());
}

#[test]
fn case_insensitive_imperative_matching() {
    let src = "YOU MUST be helpful. YOU SHOULD be accurate. ALWAYS verify facts. \
               NEVER speculate. DO NOT guess. DON'T assume. PLEASE be professional. \
               ENSURE accuracy. MAKE SURE you check. REMEMBER to cite. BE SURE to verify.";
    let doc = make_system_doc(src, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("imperative"));
}

#[test]
fn both_gates_fire_shows_both_reasons() {
    let long_and_imperative = format!(
        "{} {}",
        "word ".repeat(3000),
        "You must help. You should check. Always verify. Never assume. \
         Please be clear. Ensure accuracy. Do not guess. Don't speculate. Make sure. Remember to check. Be sure to verify."
    );
    let doc = make_system_doc(&long_and_imperative, Role::System);
    let rule = SystemMessageOverloaded;
    let ctx = Ctx {
        target_models: &[],
        max_tokens_budget: None,
    };
    let diags = rule.check(&doc, &ctx);
    assert_eq!(diags.len(), 1);
    // Both reasons should appear in the message
    assert!(diags[0].message.contains("tokens"));
    assert!(diags[0].message.contains("imperative"));
}
