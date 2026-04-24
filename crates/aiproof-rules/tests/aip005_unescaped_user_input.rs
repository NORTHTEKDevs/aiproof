mod helpers;

use aiproof_rules::rules::aip005_unescaped_user_input::UnescapedUserInput;
use helpers::run_rule;

#[test]
fn flags_unescaped_interpolation() {
    let src = "---\nrole: system\n---\nAnswer the question: {query}";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AIP005");
}

#[test]
fn clean_with_xml_delimiter() {
    let src = "---\nrole: system\n---\nAnswer this: <user_input>{query}</user_input>";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_backtick() {
    let src = "---\nrole: system\n---\nOutput: `{query}` exactly as provided.";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn ignores_user_role() {
    let src = "---\nrole: user\n---\nAnswer the question: {query}";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn flags_jinja_style() {
    let src = "---\nrole: system\n---\nQuestion: {{ user_input }} please";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AIP005");
}

#[test]
fn multiple_unescaped() {
    let src = "---\nrole: system\n---\n{a} and then {b} and finally {c}";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert_eq!(diags.len(), 3);
}

#[test]
fn clean_with_bracket() {
    let src = "---\nrole: system\n---\nQuestions: [{q}] to parse";
    let diags = run_rule(UnescapedUserInput, src, "md");
    assert!(diags.is_empty());
}
