mod helpers;

use aiproof_rules::rules::aip004_contradictory_tone::ContradictoryTone;
use helpers::run_rule;

#[test]
fn flags_concise_vs_detailed() {
    let src = "Be concise and detailed in your response.";
    let diags = run_rule(ContradictoryTone, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP004");
}

#[test]
fn clean_single_tone() {
    let src = "Be concise and clear.";
    let diags = run_rule(ContradictoryTone, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn flags_casual_vs_formal() {
    let src = "Use casual language but maintain a formal tone.";
    let diags = run_rule(ContradictoryTone, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP004");
}

#[test]
fn case_insensitive() {
    let src = "Be BRIEF. Provide THOROUGH explanations.";
    let diags = run_rule(ContradictoryTone, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP004");
}
