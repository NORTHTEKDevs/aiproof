# Contributing to aiproof

Contributions are welcome — especially new rules, false-positive reports
from real-world prompts, parser improvements, and bindings for additional
ecosystems.

## Quick start

```bash
git clone https://github.com/Frostbyte-Devs/aiproof.git
cd aiproof

# Build
cargo build --workspace

# Run the full test suite (~170 tests)
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Try the binary locally
cargo run --release -p aiproof-cli -- --format pretty path/to/your/prompts
```

Requires Rust 1.85+ and Python 3.9+ (only for the wheel build).

## Repo layout

```
crates/
├── aiproof-core    — Document, Rule trait, Diagnostic, Severity, Span
├── aiproof-parse   — per-format parsers + SDK call-site extractor
├── aiproof-rules   — the 20 bundled rules (one file per rule)
├── aiproof-config  — .aiproofrc + pyproject.toml loader
├── aiproof-report  — pretty / JSON / SARIF renderers
├── aiproof-cli     — clap CLI + file discovery + orchestration
└── aiproof-py      — pyo3 + maturin Python wheel (built separately)

fixtures/corpus/    — pinned real-world repos + baseline JSON per repo
docs/rules/         — one Markdown file per AIPxxx rule (bundled into --explain)
docs/plans/         — design doc + implementation plan
scripts/            — corpus sync + baseline generation
```

## Adding a rule

1. **Create the rule module** at
   `crates/aiproof-rules/src/rules/aipXXX_your_rule.rs`. Use an existing
   rule as a template — `aip006_hardcoded_credential.rs` for autofix
   shape, `aip001_conflicting_instructions.rs` for detection-only shape.

2. **Register it** in `crates/aiproof-rules/src/rules/mod.rs`:

   ```rust
   pub mod aipXXX_your_rule;
   // ... in register_all:
   aipXXX_your_rule::register(out);
   ```

3. **Write tests** at `crates/aiproof-rules/tests/aipXXX_your_rule.rs`
   with at least **one positive and one negative case**. Start with
   `mod helpers;` and `use helpers::run_rule;`.

4. **Write the doc** at `docs/rules/AIPXXX.md` (under 100 words:
   What / Why it matters / Example / Fix).

5. **Run the corpus regression** locally to check false-positive impact:

   ```bash
   ./scripts/sync_corpus.sh          # first run, may take several minutes
   ./scripts/generate_baselines.sh   # ~2 min after warmup
   git diff fixtures/corpus/*.baseline.json
   ```

   If your rule causes large regressions on real repos, consider
   tightening the detection or gating on `util::is_prompt_shaped(doc)`.

6. **Open a PR.** CI runs the full test suite and corpus regression.

### Rule quality bar

Rules must:
- **Emit at most one diagnostic per distinct issue** — no per-character
  duplicates.
- **Hit a ≤5 % false-positive budget** on the 20-repo corpus.
- **Gate appropriately** — rules that shouldn't fire on generic
  documentation should check `util::is_prompt_shaped(doc)` before
  scanning.
- **Be deterministic** — same input + same config = same output.
- **Pass `cargo clippy -D warnings`** and `cargo fmt --check`.

## Reporting a false positive

File an issue with:
- The exact source fragment that triggers the FP
- The rule code (e.g. `AIP011`)
- Your config (if any): `.aiproofrc` or CLI flags
- `aiproof --version`

FP reports drive prioritization for the v0.1.x patch releases.

## Adding a parser

Parsers live in `crates/aiproof-parse/src/`. Each exposes
`pub fn parse(path: &Path, source: &str) -> anyhow::Result<Vec<Document>>`.
Register it in the `parse_file` dispatcher in `src/lib.rs` keyed by file
extension.

Tests go inline in the parser file (`#[cfg(test)] mod tests`). Use
`insta::assert_yaml_snapshot` for anything with structured output.

## Coding style

- Rust 2024 edition, MSRV 1.85.
- No new dependencies without discussion — keep the binary small.
- Prefer `thiserror` in libraries, `anyhow` in binaries.
- One logical change per commit. Conventional Commits preferred
  (`feat:`, `fix:`, `docs:`, `test:`, `ci:`, `chore:`).

## Licensing

All contributions are licensed under Apache-2.0 (the project license).
By submitting a PR, you agree to license your contribution under these
terms. No CLA needed.
