use aiproof_rules::all_rules;

#[test]
fn registry_contains_20_rules() {
    let rules = all_rules();
    assert_eq!(
        rules.len(),
        20,
        "expected all 20 v0 rules registered; got {}",
        rules.len()
    );

    // Each code should be unique.
    let mut codes: Vec<&str> = rules.iter().map(|r| r.code()).collect();
    codes.sort();
    let original_len = codes.len();
    codes.dedup();
    assert_eq!(
        codes.len(),
        original_len,
        "duplicate rule codes detected: {:#?}",
        codes
    );

    // Codes should cover AIP001..=AIP020.
    let expected: Vec<String> = (1..=20).map(|i| format!("AIP{i:03}")).collect();
    let actual_codes: Vec<String> = rules.iter().map(|r| r.code().to_string()).collect();
    for e in &expected {
        assert!(
            actual_codes.contains(e),
            "missing rule code {e}: have {:#?}",
            actual_codes
        );
    }
}
