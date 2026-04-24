mod helpers;

use aiproof_rules::rules::aip002_ambiguous_output_format::AmbiguousOutputFormat;
use helpers::run_rule;

#[test]
fn flags_json_without_schema() {
    let src = "Return your answer in JSON.";
    let diags = run_rule(AmbiguousOutputFormat, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP002");
}

#[test]
fn clean_with_json_example() {
    let src = "Return JSON like: {\"answer\": \"42\"}";
    let diags = run_rule(AmbiguousOutputFormat, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_xml_example() {
    let src = "Return XML. Example:\n<answer>42</answer>";
    let diags = run_rule(AmbiguousOutputFormat, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn flags_yaml_without_schema() {
    let src = "Respond using YAML.";
    let diags = run_rule(AmbiguousOutputFormat, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP002");
}

#[test]
fn clean_with_code_fence() {
    let src = "Return CSV format.\n```\nname,age\nAlice,30\n```";
    let diags = run_rule(AmbiguousOutputFormat, src, "md");
    assert!(diags.is_empty());
}
