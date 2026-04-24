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
