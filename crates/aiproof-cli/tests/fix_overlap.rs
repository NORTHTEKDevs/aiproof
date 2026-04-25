//! Defense-in-depth regression: --fix should never panic or corrupt files
//! when multiple autofixes target overlapping spans.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn fix_handles_utf8_correctly() {
    // A non-ASCII prompt with a hardcoded credential. AIP006's autofix should
    // redact only the credential without corrupting the surrounding bytes.
    let dir = tempdir().unwrap();
    let file = dir.path().join("p.prompt.md");
    fs::write(
        &file,
        "Tu es un assistant utile. Voici la clé API: sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234.",
    )
    .unwrap();

    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--fix", "--color", "never"])
        .arg(dir.path())
        .assert()
        .code(0);

    let after = fs::read_to_string(&file).unwrap();
    assert!(
        after.contains("***REDACTED***"),
        "credential not redacted: {after:?}"
    );
    assert!(
        after.starts_with("Tu es un assistant utile."),
        "preamble corrupted: {after:?}"
    );
    assert!(after.contains("clé"), "non-ASCII char lost: {after:?}");
}
