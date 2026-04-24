# aiproof — Design Document

- Date: 2026-04-23
- Author: Kristian Baer (Northtek)
- Status: approved (brainstorming phase complete)
- License: Apache-2.0

## 1. Problem

AI developers have no static analyzer for prompts. Runtime eval tools
(Promptfoo, Braintrust, RAGAS) require LLM calls and are slow and
expensive. Runtime firewalls (Lakera, Rebuff) are hosted services focused
on attack-time detection. Nobody ships a pre-LLM, offline, editor-grade
linter that tells a developer "this prompt has these 6 issues" before a
single token is spent.

aiproof fills that gap. It is ESLint/ruff for prompts: a static analyzer
that reads prompt text (plain, markdown, Jinja, Mustache, YAML, JSON,
and prompts extracted from Python/TS source) and reports issues across
six categories — clarity, security, efficiency, behavior, portability,
best-practice — with zero LLM calls, zero network, zero cost per run.

## 2. Success criteria for v0

A v0 release is successful if all six gates pass:

1. Running `aiproof .` against 20 public open-source AI projects (pinned
   by SHA in `fixtures/corpus/`) surfaces >= 3 distinct issue types per
   project on average.
2. Human-reviewed false-positive rate across the corpus is < 10%.
3. CLI completes in < 50 ms per prompt file after warmup.
4. `pip install aiproof` completes in < 30 s on macOS, Linux, Windows.
5. VS Code extension (v0.2, post-launch) shows inline diagnostics
   within 100 ms of file save. Not in the v0 gate.
6. Deterministic output across runs (byte-for-byte stable for a given
   input and config).

## 3. Non-goals

- Runtime evaluation or LLM calls (ever).
- Multi-language prompt support beyond English (v1+).
- Embedding-based or ML-powered rules (v1+).
- Hosted dashboard or cloud features (v2 or never).
- JS/TS npm bindings (v0.3; Python wheel suffices for v0).
- VS Code extension (v0.2, ~1 week after core stable).
- GitHub Action (v0.2).

## 4. Architecture

Rust 2024-edition workspace, `crates/` layout mirroring kryos-lang.

```
aiproof/
  Cargo.toml                      # workspace root
  crates/
    aiproof-core/                 # Document, Rule trait, Diagnostic, Severity, Span
    aiproof-parse/                # per-format parsers; SDK call-site extractor
    aiproof-rules/                # bundled rules (20 at v0), one file per rule
    aiproof-config/               # .aiproofrc and pyproject.toml [tool.aiproof]
    aiproof-report/               # pretty (codespan-reporting), json, sarif
    aiproof-cli/                  # bin; clap CLI; discovery + orchestration
    aiproof-py/                   # pyo3 + maturin wheel shim
  docs/
    rules/                        # AIP001.md .. AIP020.md (--explain source)
    plans/                        # this file + implementation plan
  fixtures/                       # real-world prompts for tests and corpus
  .github/workflows/              # release, wheels, SARIF self-test
```

### 4.1 Parser strategy (hybrid, surface-matched)

| Format                                  | Parser                     | Reason                                    |
| --------------------------------------- | -------------------------- | ----------------------------------------- |
| Python (call-site + f-string extraction)| `tree-sitter-python`       | Battle-tested AST; essential for SDK rule |
| TypeScript (template literals, SDK)     | `tree-sitter-typescript`   | Same reasoning                            |
| Markdown (`.md`, `.prompt.md`)          | `tree-sitter-md`           | Frontmatter + fenced block extraction     |
| YAML (config, Prompty files)            | `serde_yaml`               | Struct-deserialize, simpler than AST walk |
| JSON (MCP schemas)                      | `serde_json`               | Same                                      |
| Jinja2                                  | hand-rolled, `logos` lexer | Tiny grammar; grammars unmaintained       |
| Mustache                                | hand-rolled, `logos` lexer | Even smaller grammar                      |
| Plain text / `.prompt`                  | raw string + line index    | Most rules are string + regex             |

Each parser returns a `Document` whose variants carry both a
format-specific AST and a normalized `PromptText` view (flat string +
line/column index). Most rules operate on the normalized view; a
minority (SDK extractor, cache-unfriendly-structure) inspect the AST.

### 4.2 File discovery (SDK-aware, three tiers)

1. **Declared.** Any glob in `.aiproofrc` or `pyproject.toml [tool.aiproof]`.
2. **Known-safe extensions and directories.** Linted automatically
   with no config. Includes: `*.prompt.md`, `*.prompt`, `*.j2`,
   `*.jinja`, `*.jinja2`, `*.mustache`, files under `prompts/**`,
   `templates/**`, `system_prompts/**`.
3. **SDK-aware extraction.** For `.py` and `.ts`/`.tsx`, tree-sitter
   walks the AST and extracts string literals passed to known call
   sites:
    - `client.messages.create(system=..., messages=[...])` (Anthropic)
    - `openai.chat.completions.create(messages=[...])` (OpenAI)
    - `client.chat.completions.create(...)` variants
    - `PromptTemplate(template=..., input_variables=...)` (LangChain)
    - `ChatPromptTemplate.from_messages([...])` (LangChain)
    - `Agent(system=..., ...)` (common agent SDK shapes)

   The extractor yields a `(file_path, span, role, text)` tuple to the
   rule engine, so rules operate on the real prompt string, not the
   surrounding Python.

### 4.3 Rule trait (sketch)

Actual signatures finalized in the implementation plan; this is for
the design record.

```rust
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;          // "AIP007"
    fn name(&self) -> &'static str;          // "missing-input-boundaries"
    fn category(&self) -> Category;
    fn severity(&self) -> Severity;
    fn check(&self, doc: &Document, ctx: &Ctx) -> Vec<Diagnostic>;
    fn autofix(&self, d: &Diagnostic, doc: &Document) -> Option<Fix> {
        let _ = (d, doc);
        None
    }
}
```

`Severity` values: `Error`, `Warning`, `Info`. Exit codes: `2`, `1`,
`0`. Exit code is the max severity observed across the run.

`Diagnostic` carries a `Span`, a primary message, an optional
`--explain`-URL, and zero or more labels that annotate contextual
spans (e.g. "this rule conflicts with the sentence above").

### 4.4 Rule authoring

Rust-only for v0. Each rule lives in its own file under
`crates/aiproof-rules/src/rules/AIPxxx_*.rs`, registered in a single
`bundle.rs` that assembles `Vec<Box<dyn Rule>>`. A YAML rule DSL
remains possible for v1 if contributor volume demands it, but is not
built now. Reason: ruff proved Rust-only rules scale to hundreds of
contributors; a DSL is a project unto itself and drags the v0 timeline.

### 4.5 Configuration

Precedence (highest wins):

1. CLI flag (`--select`, `--ignore`, `--target-model`, ...)
2. `.aiproofrc` in nearest ancestor directory (TOML)
3. `[tool.aiproof]` table in `pyproject.toml`
4. Built-in defaults

Keys at v0:

```toml
include       = ["prompts/**/*.md", "src/**/*.py"]   # globs
exclude       = ["tests/fixtures/**"]
select        = ["AIP*"]                              # enable-list
ignore        = ["AIP019"]                            # disable-list
target_models = ["claude-4.7-opus", "gpt-4"]          # scopes portability rules
max_tokens_budget = 4000                              # used by AIP011
fix           = false                                 # autofix toggle
unsafe_fixes  = false                                 # opts into risky autofixes
```

### 4.6 Output formats

- `pretty` (default, TTY): `codespan-reporting` with color, line
  numbers, squiggles, context lines, a "why this matters" one-liner,
  and a link (or `aiproof --explain AIPxxx` hint) per finding.
- `json`: stable schema, one finding per object. For editor integrations.
- `sarif`: SARIF 2.1.0 output compatible with GitHub Code Scanning.

All three are byte-for-byte deterministic given the same input + config
(critical for CI diffing). Pretty output with color disabled is stable
across terminal widths (wrap is disabled for test mode via
`--no-wrap`).

### 4.7 Autofix

Each rule may declare a safe autofix that cannot change program
meaning (e.g. redacting a hardcoded credential, wrapping user
interpolation in `<user_input>` tags, removing an unused template
variable). `--fix` applies safe fixes. `--fix --unsafe-fixes`
additionally applies rule-declared unsafe fixes. Fix application is
idempotent: running `aiproof --fix` twice is a no-op on the second
run.

### 4.8 Distribution

- `crates.io`: `aiproof-core`, `aiproof-parse`, `aiproof-rules`,
  `aiproof-report`, `aiproof-config`, `aiproof-cli` (binary).
- `PyPI`: `aiproof` wheel built via `maturin` + GitHub Actions matrix
  (manylinux2014 x86_64 / aarch64, macos-universal2,
  windows-msvc-x86_64).
- GitHub Releases: prebuilt `aiproof` binaries for the three OS x two
  arch combinations, plus source tarball and SBOM.
- Version scheme: SemVer. v0.y.z until API stabilizes; 1.0 at
  post-launch when 90-day FP data is in.

## 5. v0 rule set (20 rules)

Codes AIP001–AIP020. Each rule must be validated against at least
three real open-source prompt fixtures (both positive and negative
cases) before shipping. Three reserve rules remain available to
promote if any of the below fails the FP gate during implementation.

### Clarity

| Code   | Name                       | One-liner                                                                  |
| ------ | -------------------------- | -------------------------------------------------------------------------- |
| AIP001 | conflicting-instructions   | Two instructions that cannot both be satisfied.                            |
| AIP002 | ambiguous-output-format    | Prompt asks for JSON/XML/YAML without a schema or example.                 |
| AIP003 | undefined-role             | References "you are an X" followed by conflicting role language.           |
| AIP004 | contradictory-tone         | Both "concise" and "detailed/thorough/comprehensive" demanded.             |

### Security

| Code   | Name                       | One-liner                                                                  |
| ------ | -------------------------- | -------------------------------------------------------------------------- |
| AIP005 | unescaped-user-input       | User interpolation appears inside a system-role prompt.                    |
| AIP006 | hardcoded-credential       | API-key shape (sk-..., xoxb-..., AKIA..., ghp_..., etc.) in prompt text.   |
| AIP007 | missing-input-boundaries   | No delimiter (tag, fence, triple-quote) around interpolated user content.  |
| AIP008 | known-jailbreak-pattern    | DAN variants, role-hijack, "ignore previous instructions" signatures.      |

### Efficiency

| Code   | Name                         | One-liner                                                                |
| ------ | ---------------------------- | ------------------------------------------------------------------------ |
| AIP009 | cache-unfriendly-structure   | Variable content before stable prefix defeats Anthropic prompt caching.  |
| AIP010 | redundant-instruction        | Near-duplicate sentences within the same prompt.                         |
| AIP011 | excessive-tokens             | Prompt exceeds `max_tokens_budget` or is >= 10x the median in the repo.  |
| AIP012 | unused-template-variable     | Template variable declared but never referenced.                         |

### Behavior

| Code   | Name                         | One-liner                                                                |
| ------ | ---------------------------- | ------------------------------------------------------------------------ |
| AIP013 | missing-format-example       | JSON/XML/YAML output requested without a concrete example block.         |
| AIP014 | undefined-tool-reference     | Prompt mentions a tool name not present in the tool registry/file.       |
| AIP015 | unhandled-placeholder        | TODO, FIXME, `XXX`, `...` left inside a prompt body.                     |

### Portability

| Code   | Name                              | One-liner                                                           |
| ------ | --------------------------------- | ------------------------------------------------------------------- |
| AIP016 | claude-specific-tags-on-gpt       | `<thinking>`, `<scratchpad>` used when `target_models` includes GPT.|
| AIP017 | system-message-mismatch           | Prompt assumes system role shape that Gemini/other model handles differently. |
| AIP018 | temperature-determinism-mismatch  | Prompt demands deterministic output but API call's temperature is > 0.3. |

### Best practice

| Code   | Name                              | One-liner                                                           |
| ------ | --------------------------------- | ------------------------------------------------------------------- |
| AIP019 | missing-few-shot-for-reasoning    | Chain-of-thought / reasoning prompt without concrete examples.      |
| AIP020 | system-message-overloaded        | Single system message exceeds ~1500 tokens or bundles >8 instruction clusters. |

### Reserve

| Code   | Name                            | Promotes if...                                                      |
| ------ | ------------------------------- | ------------------------------------------------------------------- |
| AIP021 | xml-vs-markdown-mismatch        | Another rule fails FP gate.                                         |
| AIP022 | no-output-constraint            | Another rule fails FP gate.                                         |
| AIP023 | deprecated-model-id             | Another rule fails FP gate.                                         |

## 6. "User friendly" contract

Explicit UX requirements, testable during implementation:

1. `aiproof .` works with zero config. No "no prompts found" on any of
   the 20 corpus repos.
2. First paint (first diagnostic printed) < 100 ms cold on a
   20-prompt repo.
3. Per-file analysis < 50 ms after warmup, measured via `criterion`.
4. Diagnostic output uses line numbers, squiggles, color, surrounding
   context, and a one-line "why this matters" per finding.
5. `aiproof --explain AIPxxx` prints the full bundled doc (no network).
6. `aiproof --fix` is idempotent and touches only safe-fix rules.
   `--fix --unsafe-fixes` opts into the rest.
7. `pip install aiproof` works on macOS (Intel + Apple Silicon),
   Linux (x86_64 + aarch64), and Windows x86_64 from v0 day one.
8. `aiproof --init` prints a one-paste pre-commit hook snippet and a
   starter `.aiproofrc`.
9. Config errors are human: malformed TOML points to the offending
   key with a squiggle, not a serde stack trace.
10. Output is deterministic byte-for-byte across runs with identical
    input and config.

## 7. Testing strategy

- **Unit.** Each rule has snapshot tests via `insta` against fixture
  prompts covering positive and negative cases.
- **Integration.** `aiproof-cli` is executed via `assert_cmd` against
  multi-file fixtures; stdout / exit code / json / sarif are
  snapshot-compared.
- **Corpus.** 20 open-source AI projects cloned at pinned SHAs into
  `fixtures/corpus/`. Each repo has a baseline snapshot of aiproof
  output. CI fails on any unreviewed diff.
- **False-positive budget.** Every rule declares an `fp_budget.toml`
  ceiling (initial: 5% per rule). CI fails if corpus regression
  crosses the budget.
- **Performance.** `criterion` benchmarks on a 10-prompt representative
  suite. CI fails on > 10% regression.
- **Binary size.** CI fails if release binary grows by more than 15%
  between releases.
- **Determinism.** CI runs the corpus twice and diffs outputs; any
  diff fails.

## 8. Risks and open questions

1. **False-positive gate.** The hardest constraint. Mitigation: keep
   rule count low (20), validate each against 3 real prompts before
   ship, budget enforcement in CI. If a rule can't meet FP budget,
   demote it to the reserve pool and promote a reserve rule.
2. **Extraction brittleness.** SDK call-site detection will miss
   patterns (wrapper functions, dynamic prompt construction).
   Acceptable: better to under-extract with low FP than over-extract
   with many FPs. Users can always add explicit globs.
3. **Tree-sitter grammar drift.** Pin grammar versions in Cargo.toml.
   Update via explicit PRs with corpus rerun.
4. **Windows wheel build.** maturin + windows-msvc matrix can be
   flaky; reserve a day for wheel pipeline troubleshooting.
5. **HN launch fit.** The v0 demo narrative is "aiproof found N real
   issues in langchain/anthropic-cookbook/etc." This only works if
   the corpus actually surfaces findings. If corpus runs produce
   < 3 issues per repo on average, delay launch and broaden rules.

## 9. Out-of-scope for v0 (parked)

- VS Code extension (v0.2).
- GitHub Action (v0.2).
- MCP server wrapping aiproof (v0.3).
- npm / JS bindings (v0.3).
- YAML rule DSL (v1+).
- Embedding / semantic rules (v1+).
- Multi-language prompts (v1+).
- Hosted cloud features (v2 or never).

## 10. Next

Invoke `superpowers:writing-plans` to produce the task-by-task
implementation plan alongside this document at
`docs/plans/2026-04-23-aiproof-implementation-plan.md`.
