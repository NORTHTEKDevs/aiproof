mod helpers;

use aiproof_rules::rules::aip001_conflicting_instructions::ConflictingInstructions;
use helpers::run_rule;

#[test]
fn flags_json_vs_reasoning() {
    let src = "You will only output JSON. Explain your reasoning before answering.";
    let diags = run_rule(ConflictingInstructions, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP001");
}

#[test]
fn clean_when_no_contradiction() {
    let src = "Be helpful. Be concise.";
    let diags = run_rule(ConflictingInstructions, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn case_insensitive() {
    let src = "RESPOND IN JSON. Use natural language.";
    let diags = run_rule(ConflictingInstructions, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP001");
}
