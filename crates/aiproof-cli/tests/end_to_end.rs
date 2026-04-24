use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn finds_hardcoded_credential_in_markdown_prompt() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("prompt.md");
    fs::write(
        &file,
        "Your API key is sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234",
    )
    .unwrap();
    Command::cargo_bin("aiproof")
        .unwrap()
        .arg("--format")
        .arg("json")
        .arg("--color")
        .arg("never")
        .arg(dir.path())
        .assert()
        .code(2)
        .stdout(predicate::str::contains("AIP006"));
}

#[test]
fn clean_prompt_exits_zero() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("clean.prompt.md");
    fs::write(&file, "Be helpful and accurate. Respond concisely.").unwrap();
    Command::cargo_bin("aiproof")
        .unwrap()
        .arg("--format")
        .arg("json")
        .arg("--color")
        .arg("never")
        .arg(dir.path())
        .assert()
        .code(0);
}

// --explain tests

#[test]
fn explain_known_code() {
    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--explain", "AIP006"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("hardcoded-credential"));
}

#[test]
fn explain_lowercase_code_normalized() {
    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--explain", "aip001"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("AIP001"));
}

#[test]
fn explain_unknown_code_errors() {
    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--explain", "AIP999"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no rule"));
}

// --init tests

#[test]
fn init_prints_starter_config_and_precommit() {
    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--init"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(".aiproofrc"))
        .stdout(predicate::str::contains("pre-commit"));
}

// --fix tests

#[test]
fn fix_redacts_hardcoded_credential_and_is_idempotent() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("prompt.md");
    fs::write(
        &file,
        "Your API key is sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234",
    )
    .unwrap();

    // First run — should redact.
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
        !after.contains("sk-ant-api03"),
        "original credential leaked: {after:?}"
    );

    // Second run — should be a no-op.
    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--fix", "--color", "never"])
        .arg(dir.path())
        .assert()
        .code(0);

    let after2 = fs::read_to_string(&file).unwrap();
    assert_eq!(after, after2, "--fix should be idempotent");
}

#[test]
fn fix_on_clean_file_is_noop() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("clean.md");
    let content = "Be helpful and accurate. Respond concisely.";
    fs::write(&file, content).unwrap();

    Command::cargo_bin("aiproof")
        .unwrap()
        .args(["--fix", "--color", "never"])
        .arg(dir.path())
        .assert()
        .code(0)
        .stdout(predicate::str::contains("0 fixes applied"));

    let after = fs::read_to_string(&file).unwrap();
    assert_eq!(after, content, "clean file should not be modified");
}
