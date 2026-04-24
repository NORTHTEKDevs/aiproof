mod helpers;

use aiproof_core::rule::Rule;
use aiproof_rules::rules::aip007_missing_input_boundaries::MissingInputBoundaries;
use helpers::run_rule;
use std::path::PathBuf;

#[test]
fn flags_unescaped_single_brace() {
    let src = "---\nrole: system\n---\nAnswer this: {q} and provide reasoning.";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP007");
}

#[test]
fn clean_with_xml_wrapper() {
    let src = "---\nrole: system\n---\nAnswer: <user_input>{q}</user_input>";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_backtick() {
    let src = "---\nrole: system\n---\nEcho: `{input}` verbatim.";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn ignores_user_role() {
    let src = "---\nrole: user\n---\nAnswer this: {q}";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn multiple_unescaped() {
    let src = "---\nrole: system\n---\n{a} and {b} and {c}";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert_eq!(diags.len(), 3);
}

#[test]
fn autofix_wraps_in_tags() {
    let rule = MissingInputBoundaries;
    let src = "---\nrole: system\n---\nQuestion: {query} please answer";
    let path = PathBuf::from("test.md");
    let docs = aiproof_parse::parse_file(&path, src).expect("should parse");
    let diags = docs
        .iter()
        .flat_map(|d| {
            rule.check(
                d,
                &aiproof_core::rule::Ctx {
                    target_models: &[],
                    max_tokens_budget: None,
                },
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(diags.len(), 1);
    let fix = rule.autofix(&diags[0], &docs[0]).unwrap();
    assert_eq!(fix.edits.len(), 1);
    assert_eq!(fix.edits[0].replacement, "<user_input>{query}</user_input>");
    assert!(fix.safe);
}

#[test]
fn ignores_double_brace() {
    let src = "---\nrole: system\n---\n{{ variable }} here";
    let diags = run_rule(MissingInputBoundaries, src, "md");
    assert!(diags.is_empty());
}
