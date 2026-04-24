/// Print starter config and pre-commit snippet.
pub const STARTER_AIPROOFRC: &str = r#"# aiproof config — see https://aiproof.dev/config

# Files to lint. Defaults cover common locations; override for custom layouts.
# include = ["prompts/**/*.md", "src/**/*.py"]

# Files to skip. Default excludes cover common sources of false positives:
# long design/planning docs, release notes, and test fixtures/cassettes that
# intentionally embed prompt-shaped content.
exclude = [
    "docs/plans/**",
    "releasenotes/**",
    "fixtures/**",
    "tests/cassettes/**",
    "tests/recordings/**",
    "tests/fixtures/**",
    "node_modules/**",
    "target/**",
    ".venv/**",
]

# Disable noisy rules if needed.
# ignore = ["AIP019"]

# Target models enable portability rules (AIP016-AIP018).
target_models = []

# Approximate max tokens per prompt. AIP011 uses this.
max_tokens_budget = 4000
"#;

pub const PRE_COMMIT_SNIPPET: &str = r#"# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: aiproof
        name: aiproof
        entry: aiproof
        language: system
        pass_filenames: true
        types_or: [markdown, python, yaml, json]
"#;

pub fn run_init() -> i32 {
    println!("# --- Starter .aiproofrc (save to repo root) ---");
    println!("{STARTER_AIPROOFRC}");
    println!("# --- Pre-commit hook snippet (append to .pre-commit-config.yaml) ---");
    println!("{PRE_COMMIT_SNIPPET}");
    println!("# For GitHub Actions integration, see https://aiproof.dev/ci");
    0
}
