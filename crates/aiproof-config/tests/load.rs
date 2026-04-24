use aiproof_config::load;
use std::fs;
use tempfile::tempdir;

#[test]
fn loads_default_when_no_files() {
    let dir = tempdir().unwrap();
    let cfg = load(dir.path()).unwrap();
    assert!(cfg.include.is_empty());
    assert!(!cfg.fix);
}

#[test]
fn loads_aiproofrc() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(".aiproofrc"),
        r#"
include = ["prompts/**/*.md"]
target_models = ["claude-4.7-opus"]
max_tokens_budget = 5000
"#,
    )
    .unwrap();
    let cfg = load(dir.path()).unwrap();
    assert_eq!(cfg.include, vec!["prompts/**/*.md"]);
    assert_eq!(cfg.target_models, vec!["claude-4.7-opus"]);
    assert_eq!(cfg.max_tokens_budget, Some(5000));
}

#[test]
fn loads_pyproject_tool_table() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        r#"
[project]
name = "example"

[tool.aiproof]
ignore = ["AIP019"]
fix = true
"#,
    )
    .unwrap();
    let cfg = load(dir.path()).unwrap();
    assert_eq!(cfg.ignore, vec!["AIP019"]);
    assert!(cfg.fix);
}

#[test]
fn aiproofrc_wins_over_pyproject() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.aiproof]\nfix = true\n",
    )
    .unwrap();
    fs::write(dir.path().join(".aiproofrc"), "fix = false\n").unwrap();
    let cfg = load(dir.path()).unwrap();
    assert!(!cfg.fix);
}

#[test]
fn rejects_unknown_keys() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".aiproofrc"), "nonsense_field = 5\n").unwrap();
    let err = load(dir.path()).unwrap_err();
    let s = format!("{err}");
    assert!(s.contains("nonsense_field") || s.contains("unknown"));
}
