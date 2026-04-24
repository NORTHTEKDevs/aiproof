# aiproof v0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship aiproof v0 — a Rust static analyzer for AI prompts with 20 rules, `pip install aiproof`, and beautiful terminal diagnostics — validated on 20 real open-source AI repos with < 10% FP rate.

**Architecture:** Rust 2024-edition workspace (`crates/aiproof-core|parse|rules|config|report|cli|py`), parsers: tree-sitter for Python/TS/Markdown + hand-rolled Jinja/Mustache via `logos` + serde for YAML/JSON, Rust-only `Rule` trait, SDK-aware file discovery, `codespan-reporting` pretty output, `maturin` Python wheels, releases via GitHub Actions.

**Tech Stack:** Rust 2024, `tree-sitter`, `tree-sitter-python`, `tree-sitter-typescript`, `tree-sitter-md`, `logos`, `serde`, `serde_yaml`, `serde_json`, `toml`, `clap`, `thiserror`, `anyhow`, `tracing`, `codespan-reporting`, `insta`, `assert_cmd`, `criterion`, `pyo3`, `maturin`.

**Design doc:** `docs/plans/2026-04-23-aiproof-design.md` (read first).

**Reference projects:** `~/projects/active/kryos-lang/compiler/` for workspace conventions.

**Conventions:**
- Every task is TDD: write failing test → verify failure → implement → verify pass → commit.
- Commits use Conventional Commits (`feat:`, `fix:`, `test:`, `docs:`, `chore:`, `perf:`).
- Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` before every commit.
- Rust edition 2024, MSRV 1.80, strict clippy.
- **USER-WRITE** tasks are points where Kristian writes 5–10 lines himself; they are called out explicitly and the surrounding scaffold is prepared first.

---

## Phase 0 — Workspace foundation

### Task 0.1: Workspace Cargo.toml + LICENSE + README skeleton

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `LICENSE` (Apache-2.0)
- Create: `README.md` (stub)
- Create: `rust-toolchain.toml`

**Step 1: Create workspace root `Cargo.toml`.**

```toml
[workspace]
resolver = "2"
members = [
    "crates/aiproof-core",
    "crates/aiproof-parse",
    "crates/aiproof-rules",
    "crates/aiproof-config",
    "crates/aiproof-report",
    "crates/aiproof-cli",
]
exclude = ["crates/aiproof-py"]  # built separately by maturin

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.80"
license = "Apache-2.0"
repository = "https://github.com/northtek/aiproof"
authors = ["Kristian Baer <kristianb43r@gmail.com>"]

[workspace.dependencies]
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }
logos = "0.14"
tree-sitter = "0.22"
tree-sitter-python = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-md = "0.2"
codespan-reporting = "0.11"
regex = "1"
once_cell = "1"
glob = "0.3"
ignore = "0.4"
insta = { version = "1", features = ["yaml"] }
assert_cmd = "2"
criterion = { version = "0.5", features = ["html_reports"] }

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
strip = "symbols"
```

**Step 2: LICENSE file.**

Drop the standard Apache-2.0 text (https://www.apache.org/licenses/LICENSE-2.0.txt) at repo root. Copyright holder: Kristian Baer / Northtek, 2026.

**Step 3: `rust-toolchain.toml`.**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

**Step 4: README stub.**

```markdown
# aiproof

A static analyzer for AI prompts. ESLint for prompts.

Status: v0 — under active development.

## Install

Coming soon: `pip install aiproof`, `cargo install aiproof-cli`.

## License

Apache-2.0.
```

**Step 5: Verify workspace resolves.**

Run: `cargo metadata --no-deps --format-version 1 > /dev/null`
Expected: exits 0, no members (no crates yet).

**Step 6: Commit.**

```bash
git add Cargo.toml LICENSE README.md rust-toolchain.toml
git commit -m "chore: scaffold workspace"
```

### Task 0.2: CI skeleton — fmt, clippy, test

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml` (stub, disabled)

**Step 1: `ci.yml`.**

```yaml
name: ci
on:
  push: { branches: [main] }
  pull_request:
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --all
  build-matrix:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --workspace
```

**Step 2: `release.yml` stub.** Empty `jobs: {}` body gated on `if: false` for now; filled in Phase 11.

**Step 3: Commit.**

```bash
git add .github/
git commit -m "ci: add fmt+clippy+test workflow"
```

---

## Phase 1 — Core types (`aiproof-core`)

### Task 1.1: `aiproof-core` crate skeleton

**Files:**
- Create: `crates/aiproof-core/Cargo.toml`
- Create: `crates/aiproof-core/src/lib.rs`

**Step 1: `Cargo.toml`.**

```toml
[package]
name = "aiproof-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
```

**Step 2: `lib.rs` with empty module declarations.**

```rust
//! aiproof-core: Document, Rule trait, Diagnostic, Severity, Span.

pub mod document;
pub mod diagnostic;
pub mod rule;
pub mod severity;
pub mod span;
```

**Step 3: Verify build.**

Run: `cargo build -p aiproof-core`
Expected: FAIL — missing module files.

**Step 4: Create stub files.**

Create `document.rs`, `diagnostic.rs`, `rule.rs`, `severity.rs`, `span.rs` each containing `// TODO`.

**Step 5: Verify build passes.**

Run: `cargo build -p aiproof-core`
Expected: PASS with warnings only.

**Step 6: Commit.**

```bash
git add crates/aiproof-core/
git commit -m "feat(core): scaffold aiproof-core crate"
```

### Task 1.2: Span type (TDD)

**Files:**
- Modify: `crates/aiproof-core/src/span.rs`
- Test: `crates/aiproof-core/src/span.rs` (inline `#[cfg(test)]`)

**Step 1: Write failing test.**

```rust
// span.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn span_covers_byte_range_and_line_col() {
        let s = "hello\nworld\n";
        let sp = Span::from_byte_range(s, 6..11);
        assert_eq!(sp.start_line, 2);
        assert_eq!(sp.start_col, 1);
        assert_eq!(sp.end_line, 2);
        assert_eq!(sp.end_col, 6);
        assert_eq!(sp.byte_range, 6..11);
    }
}
```

**Step 2: Run test — expect FAIL.**

Run: `cargo test -p aiproof-core span`
Expected: FAIL (`Span` not defined).

**Step 3: Implement `Span`.**

```rust
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub byte_range: Range<usize>,
    pub start_line: u32,   // 1-based
    pub start_col: u32,    // 1-based
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub fn from_byte_range(source: &str, byte_range: Range<usize>) -> Self {
        let (start_line, start_col) = line_col(source, byte_range.start);
        let (end_line, end_col) = line_col(source, byte_range.end);
        Self { byte_range, start_line, start_col, end_line, end_col }
    }
}

fn line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line: u32 = 1;
    let mut col: u32 = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset { break; }
        if ch == '\n' { line += 1; col = 1; } else { col += 1; }
    }
    (line, col)
}
```

**Step 4: Run test — expect PASS.**

Run: `cargo test -p aiproof-core span`

**Step 5: Commit.**

```bash
git add crates/aiproof-core/src/span.rs
git commit -m "feat(core): Span with 1-based line/col from byte range"
```

### Task 1.3: Severity enum (TDD)

**Files:**
- Modify: `crates/aiproof-core/src/severity.rs`

**Step 1: Failing test.**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exit_code_is_max_severity() {
        assert_eq!(Severity::Info.exit_code(), 0);
        assert_eq!(Severity::Warning.exit_code(), 1);
        assert_eq!(Severity::Error.exit_code(), 2);
        assert!(Severity::Error > Severity::Warning);
    }
}
```

**Step 2: Implement.**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
         serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { Info, Warning, Error }

impl Severity {
    pub fn exit_code(self) -> i32 {
        match self { Self::Info => 0, Self::Warning => 1, Self::Error => 2 }
    }
}
```

**Step 3: Run + commit.**

```bash
cargo test -p aiproof-core severity
git add crates/aiproof-core/src/severity.rs
git commit -m "feat(core): Severity enum + exit codes"
```

### Task 1.4: Document enum

**Files:**
- Modify: `crates/aiproof-core/src/document.rs`

**Step 1: Implement `Document` enum with placeholder variants.** No test yet; shape depends on parsers.

```rust
use std::path::PathBuf;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub role: Role,
    pub source: String,          // raw file content
    pub prompt: PromptText,      // normalized view for rules
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role { Unknown, System, User, Assistant, Tool }

#[derive(Debug, Clone)]
pub struct PromptText {
    pub text: String,
    pub origin_span: Option<Span>,  // where inside source this came from
}

#[derive(Debug, Clone)]
pub enum Kind {
    PlainText,
    Markdown,
    Jinja,
    Mustache,
    YamlConfig,
    JsonSchema,
    ExtractedPython { call_site: Span },
    ExtractedTypeScript { call_site: Span },
}
```

**Step 2: Compile + commit.**

```bash
cargo build -p aiproof-core
git add crates/aiproof-core/src/document.rs
git commit -m "feat(core): Document + PromptText + Role + Kind"
```

### Task 1.5: Diagnostic + Fix + Category

**Files:**
- Modify: `crates/aiproof-core/src/diagnostic.rs`

**Step 1: Failing test.**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    #[test]
    fn diagnostic_serializes_to_stable_json_shape() {
        let d = Diagnostic {
            code: "AIP007".into(),
            message: "missing input boundaries".into(),
            severity: crate::severity::Severity::Warning,
            category: Category::Security,
            primary: Span::from_byte_range("hello", 0..5),
            labels: vec![],
            explain_url: Some("https://aiproof.dev/rules/AIP007".into()),
            fix: None,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["code"], "AIP007");
        assert_eq!(v["severity"], "warning");
        assert_eq!(v["category"], "security");
    }
}
```

**Step 2: Implement.**

```rust
use crate::{span::Span, severity::Severity};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub category: Category,
    pub primary: Span,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Label { pub span: Span, pub message: String }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<Edit>,
    pub safe: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edit { pub span: Span, pub replacement: String }

#[derive(Debug, Clone, Copy, PartialEq, Eq,
         serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Clarity,
    Security,
    Efficiency,
    Behavior,
    Portability,
    BestPractice,
}
```

**Step 3: `serde_json` is not yet a dep of aiproof-core — only needed for the test. Add as `dev-dependencies`.**

```toml
[dev-dependencies]
serde_json = { workspace = true }
```

**Step 4: Run + commit.**

```bash
cargo test -p aiproof-core diagnostic
git add crates/aiproof-core/
git commit -m "feat(core): Diagnostic, Fix, Edit, Category"
```

### Task 1.6: Rule trait + Ctx

**Files:**
- Modify: `crates/aiproof-core/src/rule.rs`

**Step 1: Implement.**

```rust
use crate::{document::Document, diagnostic::{Diagnostic, Fix, Category},
            severity::Severity};

pub struct Ctx<'a> {
    pub target_models: &'a [String],
    pub max_tokens_budget: Option<usize>,
}

pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn severity(&self) -> Severity;
    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic>;
    fn autofix(&self, diag: &Diagnostic, doc: &Document) -> Option<Fix> {
        let _ = (diag, doc);
        None
    }
}
```

**Step 2: Commit.**

```bash
cargo build -p aiproof-core
git add crates/aiproof-core/src/rule.rs
git commit -m "feat(core): Rule trait + Ctx"
```

---

## Phase 2 — Parsers (`aiproof-parse`)

### Task 2.1: Crate skeleton + plain-text parser

**Files:**
- Create: `crates/aiproof-parse/Cargo.toml`
- Create: `crates/aiproof-parse/src/lib.rs`
- Create: `crates/aiproof-parse/src/plain.rs`

**Step 1: `Cargo.toml`.**

```toml
[package]
name = "aiproof-parse"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
aiproof-core = { path = "../aiproof-core" }
logos = { workspace = true }
tree-sitter = { workspace = true }
tree-sitter-python = { workspace = true }
tree-sitter-typescript = { workspace = true }
tree-sitter-md = { workspace = true }
serde_yaml = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
regex = { workspace = true }
once_cell = { workspace = true }

[dev-dependencies]
insta = { workspace = true }
```

**Step 2: `lib.rs`.**

```rust
pub mod plain;
pub mod markdown;
pub mod jinja;
pub mod mustache;
pub mod yaml;
pub mod json_schema;
pub mod python_extract;
pub mod ts_extract;

use aiproof_core::document::Document;
use std::path::Path;

pub fn parse_file(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "md" => markdown::parse(path, source),
        "j2" | "jinja" | "jinja2" => jinja::parse(path, source),
        "mustache" => mustache::parse(path, source),
        "yaml" | "yml" => yaml::parse(path, source),
        "json" => json_schema::parse(path, source),
        "py" => python_extract::parse(path, source),
        "ts" | "tsx" => ts_extract::parse(path, source),
        _ => plain::parse(path, source),
    }
}
```

**Step 3: Plain parser.**

```rust
// plain.rs
use aiproof_core::document::{Document, Kind, PromptText, Role};
use std::path::Path;

pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>> {
    Ok(vec![Document {
        path: path.into(),
        role: Role::Unknown,
        source: source.into(),
        prompt: PromptText { text: source.into(), origin_span: None },
        kind: Kind::PlainText,
    }])
}
```

**Step 4: Stub the other module files with `pub fn parse(...)` returning empty Vec so the crate builds.**

**Step 5: Test + commit.**

```rust
// plain.rs tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plain_roundtrips_source() {
        let docs = parse(std::path::Path::new("a.prompt"), "hello").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].prompt.text, "hello");
    }
}
```

```bash
cargo test -p aiproof-parse plain
git add crates/aiproof-parse/
git commit -m "feat(parse): scaffold aiproof-parse + plain-text parser"
```

### Task 2.2: Markdown parser via tree-sitter-md

**Files:**
- Modify: `crates/aiproof-parse/src/markdown.rs`

**Step 1: Failing test using insta.**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    #[test]
    fn markdown_strips_frontmatter_and_preserves_body() {
        let src = "---\nrole: system\n---\nYou are a helpful assistant.\n";
        let docs = parse(std::path::Path::new("x.md"), src).unwrap();
        assert_eq!(docs.len(), 1);
        assert_yaml_snapshot!(docs[0].prompt.text);
        assert_eq!(docs[0].role, aiproof_core::document::Role::System);
    }
}
```

**Step 2: Implement.** Use `tree_sitter::Parser` with `tree_sitter_md::LANGUAGE`. Extract frontmatter YAML block, parse with `serde_yaml` into a minimal `struct { role: Option<String> }`, strip it from `source`, put the remainder into `PromptText`.

**Step 3: Snapshot review (first run creates `.snap.new`, review and accept via `cargo insta accept`).**

**Step 4: Commit.**

```bash
cargo test -p aiproof-parse markdown
cargo insta accept
git add crates/aiproof-parse/
git commit -m "feat(parse): markdown parser + frontmatter role extraction"
```

### Task 2.3: Jinja2 parser (hand-rolled, logos)

**Files:**
- Modify: `crates/aiproof-parse/src/jinja.rs`

**Step 1: Failing tests** — tokens `{{ var }}`, `{% block %}`, `{# comment #}`, raw text.

**Step 2: Implement `logos::Logos` enum:**

```rust
#[derive(logos::Logos, Debug, PartialEq)]
pub enum Tok {
    #[regex(r"\{\{[^}]*\}\}")] Expr,
    #[regex(r"\{%[^%]*%\}")]   Stmt,
    #[regex(r"\{#[^#]*#\}")]   Comment,
    #[regex(r"[^{]+")]         Text,
}
```

Emit `Document` with `Kind::Jinja`. Store template variable names in a side-table on the `Document` (extend `Kind::Jinja { variables: Vec<String> }`) for use by AIP012 unused-template-variable.

**Step 3: Commit.**

```bash
git add crates/aiproof-parse/src/jinja.rs crates/aiproof-core/src/document.rs
git commit -m "feat(parse): jinja2 lexer + variable table"
```

### Task 2.4: Mustache parser (hand-rolled)

**Files:**
- Modify: `crates/aiproof-parse/src/mustache.rs`

Same shape as Jinja but with `{{#section}}`, `{{^inv}}`, `{{/close}}`, `{{! comment}}`. Variable table captured the same way.

Tests for each tag type. Commit.

### Task 2.5: YAML config + JSON schema parsers

**Files:**
- Modify: `crates/aiproof-parse/src/yaml.rs`
- Modify: `crates/aiproof-parse/src/json_schema.rs`

YAML parser handles Prompty-style files (`---\nname: ...\nmodel: ...\n---\n<body>`). If no Prompty frontmatter, treat whole file as plain text inside a `YamlConfig` document.

JSON: recognise MCP server schemas (`{"type": "object", "properties": {...}}` with a `description` key) and treat descriptions as prompt text.

Tests for both. Commit.

### Task 2.6: Python SDK call-site extractor (tree-sitter)

**Files:**
- Modify: `crates/aiproof-parse/src/python_extract.rs`

**Step 1: Failing test.**

```rust
#[test]
fn extracts_anthropic_system_prompt() {
    let src = r#"
client.messages.create(
    model="claude-4.7-opus",
    system="You are a helpful assistant.",
    messages=[{"role": "user", "content": query}],
)
"#;
    let docs = parse(std::path::Path::new("a.py"), src).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].prompt.text, "You are a helpful assistant.");
    assert_eq!(docs[0].role, aiproof_core::document::Role::System);
}
```

**Step 2: Implement using tree-sitter-python.** Walk `call` nodes; match on dotted-name ends (`messages.create`, `chat.completions.create`, `PromptTemplate`, `ChatPromptTemplate.from_messages`); extract string-literal keyword arguments; handle f-strings by reconstructing the template with `{0}`, `{1}`, ... placeholders so rules see variable positions.

**Step 3: Add tests for OpenAI, LangChain PromptTemplate, ChatPromptTemplate shapes.**

**Step 4: Commit.**

```bash
cargo test -p aiproof-parse python_extract
git add crates/aiproof-parse/src/python_extract.rs
git commit -m "feat(parse): Python AST extractor for Anthropic/OpenAI/LangChain call sites"
```

### Task 2.7: TypeScript SDK call-site extractor

**Files:**
- Modify: `crates/aiproof-parse/src/ts_extract.rs`

Same approach using `tree-sitter-typescript::LANGUAGE_TSX`. Handle template literals including `${...}` interpolation. Tests for Anthropic TS SDK and OpenAI TS SDK shapes. Commit.

---

## Phase 3 — Rule engine (`aiproof-rules`)

### Task 3.1: Crate skeleton + rule registry

**Files:**
- Create: `crates/aiproof-rules/Cargo.toml`
- Create: `crates/aiproof-rules/src/lib.rs`
- Create: `crates/aiproof-rules/src/registry.rs`
- Create: `crates/aiproof-rules/src/rules/mod.rs`

**Step 1: `Cargo.toml`.**

```toml
[package]
name = "aiproof-rules"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
aiproof-core  = { path = "../aiproof-core" }
aiproof-parse = { path = "../aiproof-parse" }
regex         = { workspace = true }
once_cell     = { workspace = true }
```

**Step 2: `registry.rs`.**

```rust
use aiproof_core::rule::Rule;

pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut v: Vec<Box<dyn Rule>> = Vec::new();
    crate::rules::register_all(&mut v);
    v
}
```

**Step 3: `rules/mod.rs`.**

```rust
use aiproof_core::rule::Rule;
pub fn register_all(_out: &mut Vec<Box<dyn Rule>>) { /* filled per-rule task */ }
```

**Step 4: Commit.**

```bash
cargo build -p aiproof-rules
git add crates/aiproof-rules/
git commit -m "feat(rules): crate skeleton + registry"
```

### Task 3.2: Rule runner + snapshot harness

**Files:**
- Create: `crates/aiproof-rules/tests/helpers.rs`

A reusable test helper:

```rust
pub fn run_rule<R: aiproof_core::rule::Rule>(rule: R, src: &str, ext: &str) -> Vec<aiproof_core::diagnostic::Diagnostic> {
    let path = std::path::PathBuf::from(format!("test.{ext}"));
    let docs = aiproof_parse::parse_file(&path, src).unwrap();
    let ctx = aiproof_core::rule::Ctx { target_models: &[], max_tokens_budget: None };
    docs.iter().flat_map(|d| rule.check(d, &ctx)).collect()
}
```

Every rule task below uses this helper. Commit.

---

## Phase 4 — Rules (AIP001–AIP020)

Each rule follows the pattern:

1. Create `crates/aiproof-rules/src/rules/AIPxxx_name.rs` with a struct + `impl Rule`.
2. Add a failing `insta::assert_yaml_snapshot!` test for at least one positive and one negative case.
3. Implement; accept snapshot.
4. Register in `rules/mod.rs`.
5. Add a corresponding Markdown file under `docs/rules/AIPxxx.md` explaining the rule, why it matters, autofix notes, false-positive risks.
6. Commit.

Tasks 4.1–4.20 are one task per rule (AIP001 through AIP020). Each follows the template above. Notable specifics below.

### Task 4.1 (AIP001, conflicting-instructions)

Regex pair detection: maintain a static list of `(a, b)` imperative pairs known to conflict (e.g. `("be concise", "be thorough")`, `("only output JSON", "explain your reasoning")`). Match both in the prompt; report the pair as conflicting.

### Task 4.2 (AIP002, ambiguous-output-format)

If prompt mentions `json`/`yaml`/`xml` (case-insensitive) but contains no schema (no `{`...`}` block, no `<Example>` tag), emit diagnostic.

### Task 4.3 (AIP003, undefined-role)

Detect `you are a` followed later by contradictory role (e.g. "you are a helpful assistant" and later "act as a pirate").

### Task 4.4 (AIP004, contradictory-tone)

Pair list like AIP001 but scoped to tone descriptors (`concise`, `detailed`, `brief`, `thorough`, `comprehensive`, `terse`, `verbose`).

### Task 4.5 (AIP005, unescaped-user-input)

For `Role::System` prompts, flag interpolation (`{user_query}`, `{{ user_input }}`, etc.) that is not wrapped in XML-style tags or triple-backtick fences.

### Task 4.6 (AIP006, hardcoded-credential) — includes safe autofix

Regex list:
- `sk-[A-Za-z0-9]{20,}`
- `AKIA[0-9A-Z]{16}`
- `ghp_[A-Za-z0-9]{36}`
- `xoxb-[0-9A-Za-z-]+`
- `AIza[0-9A-Za-z_-]{35}`

Autofix: replace match with `***REDACTED***`. Mark fix `safe = true`.

### Task 4.7 (AIP007, missing-input-boundaries) — includes safe autofix

If a `Role::System` prompt interpolates user content without surrounding tags (`<user_input>...</user_input>`), emit. Autofix wraps interpolation site in `<user_input>{...}</user_input>`. Safe.

### Task 4.8 (AIP008, known-jailbreak-pattern) — **USER-WRITE**

**Context:** I'll scaffold the rule struct + test harness + 3 seed signatures (DAN, "ignore previous instructions", "you are now in developer mode"). The real signature list benefits from your security domain knowledge.

**Request:** In `crates/aiproof-rules/src/rules/AIP008_known_jailbreak_pattern.rs`, extend the `SIGNATURES: &[&str]` array with 10–15 additional jailbreak patterns you've seen in the wild or that have appeared in red-team literature. 5–10 lines.

**Why it matters:** This rule's value is entirely proportional to the quality of the signature list. Generic lists from public repos have high FP. Your curated list is the moat.

### Task 4.9 (AIP009, cache-unfriendly-structure) — **USER-WRITE threshold**

**Context:** I'll implement the AST walk that detects variable content before a large static prefix (Anthropic caching requires stable prefix ≥ 1024 tokens for cache hits on Opus, 2048 for Haiku). Signal: interpolation (`{var}`) appearing within first N tokens of a system prompt.

**Request:** Set `const PREFIX_TOKEN_THRESHOLD: usize = ?` based on your sense of your own prompts' structure. I'll scaffold with `1024`; adjust in `AIP009_cache_unfriendly_structure.rs`. 1–2 lines.

### Task 4.10 (AIP010, redundant-instruction) — includes safe autofix

MinHash-based near-duplicate detection across sentences of the same prompt. Threshold: Jaccard > 0.85. Autofix: remove the second occurrence. Safe.

### Task 4.11 (AIP011, excessive-tokens) — **USER-WRITE threshold**

**Request:** Set `DEFAULT_MAX_TOKENS: usize = ?` in `AIP011_excessive_tokens.rs`. Default in config is `max_tokens_budget = 4000`; this rule reports when a prompt exceeds that or is `>= 10x` the median prompt length in the project. Tune both constants. 2–3 lines.

### Task 4.12 (AIP012, unused-template-variable) — includes safe autofix

For `Kind::Jinja` and `Kind::Mustache`, cross-reference declared variables (from parser) against references in the template body. Report unused. Autofix: remove declaration line. Safe.

### Task 4.13 (AIP013, missing-format-example)

Prompt requests structured output (`return JSON`, etc.) but contains no example. Similar shape to AIP002.

### Task 4.14 (AIP014, undefined-tool-reference)

If a sibling `tools.json` / `tools.yaml` / `tools` section in Prompty frontmatter exists, build the tool-name set; flag references in the prompt body that don't match.

### Task 4.15 (AIP015, unhandled-placeholder)

Regex: `\bTODO\b|\bFIXME\b|\bXXX\b|\bPLACEHOLDER\b|\.\.\.TBD`. Emit as `Warning`. No autofix (ambiguous intent).

### Task 4.16 (AIP016, claude-specific-tags-on-gpt)

If `ctx.target_models` contains any GPT model and prompt contains `<thinking>`, `<scratchpad>`, or `<reflection>` tags, emit.

### Task 4.17 (AIP017, system-message-mismatch)

If `target_models` contains `gemini-*` and prompt assumes Anthropic system-role structure (first line `You are ...` followed by unguarded user instructions), emit.

### Task 4.18 (AIP018, temperature-determinism-mismatch)

Detect prompts asking for deterministic output (`always return exactly`, `must produce the same answer`) AND the extracted call site has `temperature > 0.3`. Requires SDK extractor to store `temperature` on the `Document.kind` variant. Small extension to Python/TS extractors in this task.

### Task 4.19 (AIP019, missing-few-shot-for-reasoning)

Prompt contains reasoning cues (`think step by step`, `reason through`, `chain of thought`) but no few-shot examples (no `Example:` / `Input:` / `Output:` blocks).

### Task 4.20 (AIP020, system-message-overloaded)

System prompt exceeds 1500 tokens (rough estimate: `source.split_whitespace().count() * 1.3`) OR contains > 8 imperative clauses (heuristic: count of sentences beginning with a verb followed by `,`).

---

## Phase 5 — Config (`aiproof-config`)

### Task 5.1: Crate + `.aiproofrc` loader

**Files:**
- Create: `crates/aiproof-config/Cargo.toml`
- Create: `crates/aiproof-config/src/lib.rs`

**Step 1: Config struct.**

```rust
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub select: Vec<String>,
    pub ignore: Vec<String>,
    pub target_models: Vec<String>,
    pub max_tokens_budget: Option<usize>,
    pub fix: bool,
    pub unsafe_fixes: bool,
}

pub fn load(start: &std::path::Path) -> anyhow::Result<Config> {
    // 1. Walk ancestors looking for .aiproofrc (TOML).
    // 2. If not found, check nearest pyproject.toml for [tool.aiproof].
    // 3. Return Default otherwise.
    unimplemented!()
}
```

**Step 2: Tests for all three load paths.**

**Step 3: Commit.**

### Task 5.2: Human error messages

When TOML is malformed, use `codespan-reporting` to point at the offending key with the actual file path and line. No raw serde error dumps.

---

## Phase 6 — Report sinks (`aiproof-report`)

### Task 6.1: Crate skeleton + pretty renderer

**Files:**
- Create: `crates/aiproof-report/Cargo.toml`
- Create: `crates/aiproof-report/src/lib.rs`
- Create: `crates/aiproof-report/src/pretty.rs`

Use `codespan_reporting::term` with `ColorChoice::Auto`, Unicode chars enabled when not `NO_COLOR`, Unicode chars disabled when `CI` env var set. Tests assert byte-for-byte output against `insta` snapshots with `NO_COLOR=1`.

### Task 6.2: JSON output

Stable schema:

```json
{
  "version": "0.1",
  "diagnostics": [ { "file": "...", "code": "AIP007", ... } ],
  "summary": { "error": 0, "warning": 3, "info": 1 }
}
```

Commit.

### Task 6.3: SARIF 2.1.0 output

Implement SARIF struct per spec section 3. Each rule becomes a `reportingDescriptor`. Test against the SARIF JSON schema (download once, pin locally at `fixtures/sarif-2.1.0.schema.json`). Commit.

---

## Phase 7 — CLI (`aiproof-cli`)

### Task 7.1: Crate + clap args

**Files:**
- Create: `crates/aiproof-cli/Cargo.toml`
- Create: `crates/aiproof-cli/src/main.rs`

Clap args:
- `paths: Vec<PathBuf>` (positional, default `.`)
- `--format {pretty,json,sarif}` (default `pretty`)
- `--select <CODE>...`
- `--ignore <CODE>...`
- `--target-model <NAME>...`
- `--fix` / `--unsafe-fixes`
- `--explain <CODE>` (prints bundled `docs/rules/AIPxxx.md`)
- `--init` (prints pre-commit snippet + starter `.aiproofrc`)
- `--no-color` / `--color {auto,always,never}`
- `--version` / `--help`

Commit after args compile.

### Task 7.2: File discovery (three tiers from design 4.2)

Use `ignore::WalkBuilder` so `.gitignore` is respected. For each tier:
1. Declared include globs → apply.
2. Known-safe patterns (`*.prompt.md`, `*.j2`, `*.jinja*`, `*.mustache`, `prompts/**`, `templates/**`, `system_prompts/**`).
3. `.py` and `.ts`/`.tsx` → hand to SDK extractor; produce zero or more `Document`s per file.

Tests against `fixtures/corpus/sample-repo-A/`. Commit.

### Task 7.3: Orchestration (wire config → discovery → parse → rules → report)

Load config → walk files → parse → run rules → filter by `select`/`ignore` → report.

Test end-to-end via `assert_cmd` against a fixtures directory. Commit.

### Task 7.4: `--fix` pipeline

Apply safe fixes first. For each fix, sort by span (descending) to avoid offset shift; rewrite file atomically via tempfile + rename. Re-run analysis to confirm no new diagnostics introduced; if they are, revert and print a warning.

Test: idempotency — running `--fix` twice produces identical file on second run.

### Task 7.5: `--explain` — **USER-WRITE (partial)**

Bundle `docs/rules/AIPxxx.md` files into the binary via `include_str!`. `--explain AIP007` prints the file contents.

**Request:** Write the copy for `docs/rules/AIP007.md`, `AIP008.md`, `AIP009.md`, `AIP011.md`, `AIP020.md` (5 rules) in your own voice. 50–100 words each. Sections: "What", "Why it matters", "Example", "Fix". I'll handle the remaining 15.

### Task 7.6: `--init` — **USER-WRITE**

**Request:** Write the copy printed by `aiproof --init`. It should output:
1. A sample `.aiproofrc` (5–10 lines)
2. A pre-commit hook snippet (5 lines)

The tone and wording should match how you write. Located at `crates/aiproof-cli/src/commands/init_snippets.rs` — I'll scaffold with placeholders.

---

## Phase 8 — Python bindings (`aiproof-py`)

### Task 8.1: pyo3 + maturin scaffold

**Files:**
- Create: `crates/aiproof-py/Cargo.toml`
- Create: `crates/aiproof-py/pyproject.toml`
- Create: `crates/aiproof-py/src/lib.rs`

`Cargo.toml` has `crate-type = ["cdylib"]`, depends on other workspace crates. `pyproject.toml` uses `[build-system] requires = ["maturin>=1.7"]`. `lib.rs` exposes `fn check(source: String, path: String) -> Vec<PyDiagnostic>` as a `#[pyfunction]`.

**Step 2: Build wheel locally.**

```bash
pip install maturin
cd crates/aiproof-py
maturin build --release
```

Expected: wheel produced under `target/wheels/`.

**Step 3: Commit.**

### Task 8.2: Python API surface + smoke tests

```python
import aiproof
diags = aiproof.check(open("prompt.md").read(), "prompt.md")
assert diags[0].code.startswith("AIP")
```

Add `pytest` smoke tests under `crates/aiproof-py/tests/`. Commit.

---

## Phase 9 — Corpus + baselines

### Task 9.1: Corpus harness

**Files:**
- Create: `fixtures/corpus/CORPUS.toml`
- Create: `scripts/sync_corpus.sh`

`CORPUS.toml` pins 20 open-source AI repos by `{ name, url, sha }`. `sync_corpus.sh` shallow-clones each to `fixtures/corpus/<name>/` at the pinned SHA.

Seed corpus (draft — finalize during execution):
- `langchain-ai/langchain`
- `anthropics/anthropic-cookbook`
- `openai/openai-cookbook`
- `run-llama/llama_index`
- `microsoft/autogen`
- `joaomdmoura/crewAI`
- `Significant-Gravitas/AutoGPT`
- `yoheinakajima/babyagi`
- `transitive-bullshit/chatgpt-api`
- `microsoft/prompty`
- ... 10 more to be decided.

### Task 9.2: Baseline snapshots

Run `aiproof --format json fixtures/corpus/<name>/` for each; commit output as `fixtures/corpus/<name>.baseline.json`. CI diffs against baseline and fails on unreviewed changes.

### Task 9.3: FP-budget enforcement

For every rule, define `fp_budget: 0.05` in `crates/aiproof-rules/src/rules/AIPxxx_*.rs` as `const FP_BUDGET: f32 = 0.05;`. A CI step parses the corpus baselines, counts total diagnostics per rule vs. a human-reviewed "true positive" count (maintained as a JSON file), fails if FP rate > budget.

Human review of the first corpus pass is a multi-hour task — scheduled at end of Phase 9.

---

## Phase 10 — Benchmarks + binary-size gate

### Task 10.1: criterion bench

**Files:**
- Create: `benches/single_file.rs`

Bench 10 representative prompts, target < 50 ms per file. Commit.

### Task 10.2: Binary size gate

GitHub Actions step that compares `ls -l target/release/aiproof` size against previous release; fail if > 15% growth.

---

## Phase 11 — Release pipeline

### Task 11.1: Wheel + binary matrix

Fill `release.yml` with:
- `maturin publish --target <matrix>` for manylinux (x86_64 + aarch64), macos-universal2, windows-msvc-x86_64.
- Upload prebuilt `aiproof` binary for three OS × two arch combos to GitHub Release.
- `cargo publish` for each crate in dependency order.

Test via a dry-run release on a tag like `v0.0.1-rc.1`.

### Task 11.2: README + docs site stub

Replace README stub with a launch-ready README: tagline, install, example output screenshot, rule index table linking to `docs/rules/`. No docs site yet — `docs/rules/*.md` live in the repo and are enough for v0.

Commit.

### Task 11.3: Tag v0.1.0

Once corpus baselines are green and FP budgets are met, tag `v0.1.0` and run release workflow. Post-release, monitor FP reports; patch releases via `v0.1.z` as needed.

---

## Task index (for `executing-plans` / `subagent-driven-development`)

| Task | Phase | Effort | Blockers |
|---|---|---|---|
| 0.1 | Workspace | 10 min | none |
| 0.2 | CI | 10 min | 0.1 |
| 1.1 | Core skeleton | 5 min | 0.1 |
| 1.2 | Span | 15 min | 1.1 |
| 1.3 | Severity | 10 min | 1.1 |
| 1.4 | Document | 10 min | 1.2 |
| 1.5 | Diagnostic | 15 min | 1.4 |
| 1.6 | Rule trait | 10 min | 1.5 |
| 2.1 | Parse skeleton + plain | 15 min | 1.6 |
| 2.2 | Markdown | 30 min | 2.1 |
| 2.3 | Jinja | 30 min | 2.1 |
| 2.4 | Mustache | 20 min | 2.3 |
| 2.5 | YAML + JSON | 30 min | 2.1 |
| 2.6 | Python extract | 60 min | 2.1 |
| 2.7 | TypeScript extract | 45 min | 2.6 |
| 3.1 | Rules skeleton | 15 min | 2.7 |
| 3.2 | Test harness | 15 min | 3.1 |
| 4.1–4.20 | Rules (20 rules) | ~30 min each | 3.2 |
| 5.1–5.2 | Config | 45 min | 3.2 |
| 6.1–6.3 | Report | 90 min | 1.5 |
| 7.1 | CLI args | 15 min | 5.1 |
| 7.2 | Discovery | 45 min | 7.1 |
| 7.3 | Orchestration | 45 min | 7.2, 6.1 |
| 7.4 | --fix | 30 min | 7.3 |
| 7.5 | --explain | 15 min | 7.3 |
| 7.6 | --init | 10 min | 7.3 |
| 8.1–8.2 | Python bindings | 60 min | 4.* |
| 9.1–9.3 | Corpus | 2 hr + manual review | 7.3 |
| 10.1–10.2 | Benchmarks | 30 min | 7.3 |
| 11.1–11.3 | Release | 3 hr | 9.3 |

Total: ~30–40 hours of focused work. Plan fits in a 6–8 week calendar window with launch target HN post + X thread at tag `v0.1.0`.

---

## Definition of done for v0

All six success gates from the design doc pass:

1. Corpus run surfaces >= 3 distinct issue types per repo on average.
2. Human-reviewed FP rate < 10%.
3. CLI < 50 ms per file.
4. `pip install aiproof` works on all three OSes.
5. (Not in gate) VS Code extension scheduled for v0.2.
6. Deterministic output.

Ship.
