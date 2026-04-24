mod helpers;

use aiproof_rules::rules::aip015_unhandled_placeholder::UnhandledPlaceholder;
use helpers::run_rule;

#[test]
fn flags_todo() {
    let src = "TODO: write the system prompt.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP015");
}

#[test]
fn flags_fixme() {
    let src = "FIXME: tune the temperature.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP015");
}

#[test]
fn flags_xxx() {
    let src = "XXX: this needs refactoring.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP015");
}

#[test]
fn flags_placeholder() {
    let src = "PLACEHOLDER: insert the user's name here.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP015");
}

#[test]
fn flags_tbd() {
    let src = "The policy is ...TBD.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP015");
}

#[test]
fn clean_without_markers() {
    let src = "The quick brown fox.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn multiple_markers() {
    let src = "TODO: fix this. FIXME: and that. XXX: and the other.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert_eq!(diags.len(), 3);
}

#[test]
fn case_sensitive_uppercase_only() {
    let src = "todo: this should not match.";
    let diags = run_rule(UnhandledPlaceholder, src, "md");
    assert!(diags.is_empty());
}
