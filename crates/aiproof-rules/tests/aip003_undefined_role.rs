mod helpers;

use aiproof_rules::rules::aip003_undefined_role::UndefinedRole;
use helpers::run_rule;

#[test]
fn flags_conflicting_roles() {
    let src = "You are a helpful assistant. You are a pirate.";
    let diags = run_rule(UndefinedRole, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP003");
}

#[test]
fn clean_single_role() {
    let src = "You are a helpful assistant.";
    let diags = run_rule(UndefinedRole, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_same_role_repeated() {
    let src = "You are a helpful assistant. You are a helpful assistant.";
    let diags = run_rule(UndefinedRole, src, "md");
    assert!(diags.is_empty());
}
