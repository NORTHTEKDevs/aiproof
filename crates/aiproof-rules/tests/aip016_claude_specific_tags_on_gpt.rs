mod helpers;

use aiproof_rules::rules::aip016_claude_specific_tags_on_gpt::ClaudeSpecificTagsOnGpt;
use helpers::{run_rule, run_rule_with_models};

#[test]
fn flags_thinking_tag_when_targeting_gpt() {
    let src = "Please reason inside <thinking>...</thinking> before answering.";
    let models = vec!["gpt-4".to_string()];
    let diags = run_rule_with_models(ClaudeSpecificTagsOnGpt, src, "md", &models);
    assert!(!diags.is_empty());
}

#[test]
fn clean_when_no_target_models() {
    let src = "Please reason inside <thinking>...</thinking>.";
    let diags = run_rule(ClaudeSpecificTagsOnGpt, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_when_target_is_claude_only() {
    let src = "Please reason inside <thinking>...</thinking>.";
    let models = vec!["claude-4.7-opus".to_string()];
    let diags = run_rule_with_models(ClaudeSpecificTagsOnGpt, src, "md", &models);
    assert!(diags.is_empty());
}

#[test]
fn flags_scratchpad_tag_on_gpt_target() {
    let src = "Use <scratchpad>notes here</scratchpad> for planning.";
    let models = vec!["gpt-4".to_string()];
    let diags = run_rule_with_models(ClaudeSpecificTagsOnGpt, src, "md", &models);
    assert!(!diags.is_empty());
}

#[test]
fn flags_reflection_tag_on_openai_target() {
    let src = "Please <reflection>reflect on your approach</reflection>.";
    let models = vec!["openai".to_string()];
    let diags = run_rule_with_models(ClaudeSpecificTagsOnGpt, src, "md", &models);
    assert!(!diags.is_empty());
}

#[test]
fn case_insensitive_matching() {
    let src = "Use <THINKING>uppercase tags</THINKING> here.";
    let models = vec!["gpt-4".to_string()];
    let diags = run_rule_with_models(ClaudeSpecificTagsOnGpt, src, "md", &models);
    assert!(!diags.is_empty());
}
