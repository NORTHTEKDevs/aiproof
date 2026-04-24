mod helpers;

use aiproof_rules::rules::aip019_missing_few_shot_for_reasoning::MissingFewShotForReasoning;
use helpers::run_rule;

#[test]
fn flags_step_by_step_without_examples() {
    let src = "Think step by step before answering.";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP019");
}

#[test]
fn clean_when_example_signal_present() {
    let src = "Think step by step. Example:\nQ: 1+1\nA: 2";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_when_no_reasoning_cue() {
    let src = "Just answer the question.";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_input_output_example() {
    let src = "Reason through this:\nInput: foo\nOutput: bar\nNow try: baz";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_qa_example() {
    let src = "Let's think step by step.\nQ: What is 2+2?\nA: 4";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_code_fence() {
    let src = "Show your work:\n```\nInput: 5\nOutput: 10\n```";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn clean_with_list_examples() {
    let src =
        "Work through this step by step:\n- Example 1: foo\n- Example 2: bar\n- Example 3: baz";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn case_insensitive_reasoning_cue() {
    let src = "THINK STEP BY STEP before answering.";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert_eq!(diags.len(), 1);
}

#[test]
fn detects_reason_carefully() {
    let src = "Reason carefully about the problem.";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert_eq!(diags.len(), 1);
}

#[test]
fn detects_chain_of_thought() {
    let src = "Use a chain of thought approach.";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert_eq!(diags.len(), 1);
}

#[test]
fn clean_with_example_1_and_2() {
    let src = "Let's think step by step.\nExample 1: A\nExample 2: B";
    let diags = run_rule(MissingFewShotForReasoning, src, "md");
    assert!(diags.is_empty());
}
