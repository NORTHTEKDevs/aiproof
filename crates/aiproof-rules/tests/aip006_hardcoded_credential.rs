mod helpers;

use aiproof_core::rule::Rule;
use aiproof_rules::rules::aip006_hardcoded_credential::HardcodedCredential;
use helpers::run_rule;
use std::path::PathBuf;

#[test]
fn flags_anthropic_key() {
    let src = "---\nrole: system\n---\nYour API key is sk-ant-api03-abcdefg1234567890_hijklmnop";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP006");
    assert!(diags[0].message.contains("anthropic"));
}

#[test]
fn flags_openai_key() {
    let src = "---\nrole: system\n---\nsecret: sk-1234567890abcdefghij";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, "AIP006");
}

#[test]
fn flags_aws_access_key() {
    let src = "---\nrole: system\n---\nAWS key: AKIA1234567890ABCDEF";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, "AIP006");
    assert!(diags[0].message.contains("aws-access"));
}

#[test]
fn flags_github_pat() {
    let src = "---\nrole: system\n---\ntoken is ghp_abcdefghijklmnopqrstuvwxyz123456";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("github-pat"));
}

#[test]
fn flags_slack_bot() {
    let src = "---\nrole: system\n---\nbot: xoxb-123456789-1234567890-abcdefghij";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("slack-bot"));
}

#[test]
fn flags_google_api() {
    let src = "---\nrole: system\n---\nkey: AIzaaBcDeFgHiJkLmNoPqRsTuVwXyZ12345678";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(!diags.is_empty());
    assert!(diags[0].message.contains("google-api"));
}

#[test]
fn flags_stripe_live() {
    let src = "---\nrole: system\n---\nkey: sk_live_abcdefghij1234567890abcd";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("stripe-live"));
}

#[test]
fn clean_when_no_credential() {
    let src = "---\nrole: system\n---\nThis is a normal prompt about API usage.";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(diags.is_empty());
}

#[test]
fn autofix_redacts_credential() {
    let rule = HardcodedCredential;
    let src = "---\nrole: system\n---\nYour key: sk-ant-api03-abcdefg1234567890_hijklmnop here";
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
    assert_eq!(fix.edits[0].replacement, "***REDACTED***");
    assert!(fix.safe);
}

#[test]
fn multiple_credentials() {
    let src = "---\nrole: system\n---\nanthropc: sk-ant-api03-abcdefghijklmnopqrst and aws: AKIA1234567890ABCDEF";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 2);
}

#[test]
fn skips_placeholder_xxxxx_runs() {
    // Real-world false positive: AutoGPT setup docs use this exact pattern.
    let src = "---\nrole: system\n---\nANTHROPIC_API_KEY=sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(
        diags.is_empty(),
        "5+ identical-char run should be treated as placeholder"
    );
}

#[test]
fn skips_self_labeled_random_example() {
    // Real-world false positive: haystack release notes show this in a Python example.
    let src = "---\nrole: system\n---\nSecret.from_token(\"sk-randomAPIkeyasdsa32ekasd32e\")";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(
        diags.is_empty(),
        "matched string containing 'random' should be treated as placeholder"
    );
}

#[test]
fn skips_self_labeled_your_key_example() {
    let src = "---\nrole: system\n---\nset OPENAI_API_KEY=sk-yourActualKeyGoesHereExampleValue";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert!(diags.is_empty(), "'your' label should suppress");
}

#[test]
fn still_flags_realistic_random_alphanumeric() {
    // No placeholder labels, no identical-char runs — should still flag.
    let src = "---\nrole: system\n---\nkey: sk-ant-api03-Q8mWv3kLp2xRn7tBfHj4Zd9YcGsAuP1bN5EiOoTr";
    let diags = run_rule(HardcodedCredential, src, "md");
    assert_eq!(diags.len(), 1, "non-placeholder string must still flag");
}
