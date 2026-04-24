mod helpers;

use aiproof_rules::rules::aip013_missing_format_example::MissingFormatExample;
use helpers::run_rule;

#[test]
fn flags_return_json_without_example() {
    let src = "Please return JSON.";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP013");
}

#[test]
fn clean_with_json_example_in_text() {
    let src = "Please return JSON. Example: {\"answer\": 42}";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_xml_example() {
    let src = "Respond with XML. <answer>42</answer>";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn flags_output_structured_format() {
    let src = "Output structured format.";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP013");
}

#[test]
fn clean_with_code_fence() {
    let src = "Return CSV.\n```\nname,age\nAlice,30\n```";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_for_example() {
    let src = "Produce JSON, for example {\"key\": \"value\"}";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn flags_produce_yaml() {
    let src = "Produce YAML.";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP013");
}

#[test]
fn ignores_plain_text() {
    let src = "Just write some plain text.";
    let diags = run_rule(MissingFormatExample, src, "md");
    assert!(diags.is_empty());
}
