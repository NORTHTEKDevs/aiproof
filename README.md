<div align="center">

# aiproof

**ESLint for AI prompts.** A static analyzer for the prompts you feed to
LLMs — finds real bugs before a single token is spent.

[![CI](https://github.com/Frostbyte-Devs/aiproof/actions/workflows/ci.yml/badge.svg)](https://github.com/Frostbyte-Devs/aiproof/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/aiproof-cli?label=crates.io)](https://crates.io/crates/aiproof-cli)
[![PyPI](https://img.shields.io/pypi/v/aiproof?label=PyPI)](https://pypi.org/project/aiproof/)
[![Python](https://img.shields.io/pypi/pyversions/aiproof)](https://pypi.org/project/aiproof/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

[Who this is for](#who-this-is-for) · [What aiproof is *not*](#what-aiproof-is-not) · [Install](#install) · [Quick start](#quick-start) · [Rules](#rules) · [Configuration](#configuration) · [Python API](#python-api) · [FAQ](#faq) · [Comparison](#how-is-this-different-from-)

</div>

---

## Why this exists

Every AI developer has the same daily bug:

- A prompt contradicts itself.
- A hardcoded API key leaks into a system message.
- A prompt interpolates user input without delimiters and gets jailbroken.
- A prompt requests JSON but never shows a schema, so the model returns prose.
- A system prompt places variable content before a 1024-token stable prefix
  and quietly defeats Anthropic prompt caching — doubling your bill.

Runtime tools (Promptfoo, Braintrust, Lakera) catch these *after* you make
LLM calls — which costs money, adds latency, and assumes you already have
evals set up. `aiproof` runs before a single token is spent:

- **Zero LLM calls.** All checks are pure text and AST analysis.
- **Zero network.** Works offline, air-gapped, in restricted environments.
- **Zero inference cost.** Runs in milliseconds, not seconds.
- **Twenty rules** covering clarity, security, efficiency, behavior,
  portability, and best-practice categories.

## Who this is for

`aiproof` is for developers whose **prompts live in `git`** — committed
files, version-controlled templates, or string arguments passed to LLM
SDK calls. Concretely:

- **Engineers shipping LLM-backed products.** You have
  `client.messages.create(system=...)`, `openai.chat.completions.create`,
  `PromptTemplate(...)`, or `ChatPromptTemplate.from_messages([...])` in
  your repo. Run `aiproof` as a pre-commit hook so an accidentally
  hardcoded API key, a missing input boundary, or a contradictory
  instruction never reaches `main`.

- **Prompt-engineering teams maintaining a library.** You've got
  `prompts/triage.prompt.md`, `prompts/summarize.prompt.md`,
  `prompts/escalate.prompt.md` — dozens of versioned templates. CI
  catches when a teammate's edit introduces a regression (a contradiction,
  a removed schema example, a tone clash).

- **Open-source AI library maintainers.** You've got hundreds of example
  prompts in cookbooks, READMEs, and docs. `aiproof` finds credentials
  pasted in by accident (we found two real ones in the wild — see below)
  and catches model-portability issues (Claude-specific tags in a
  GPT-targeted example, etc.).

- **Cost-conscious teams using Anthropic prompt caching.** You're
  spending real money on Claude Opus and just enabled caching. `AIP009`
  catches every system prompt that places variable content in the first
  ~1024 tokens, defeating the cache and silently doubling your bill.

- **Security / platform engineers reviewing prompt PRs.** `aiproof
  --format sarif` plugs into GitHub Code Scanning so credential leaks
  and prompt-injection vectors show up as PR comments — same workflow as
  any other security linter.

## What aiproof is *not*

This is the most common misconception, so worth being direct about it:

- **Not a prompt rewriter.** It does not make your prompt "better" or
  "more effective." It points at specific, well-defined classes of bugs
  (a hardcoded credential, a contradictory instruction, a missing
  delimiter). For a few rules it does mechanical fixes (redact the
  credential, wrap the user input in tags, dedupe a sentence). It will
  never restructure your prompt for clarity. For prompt optimization,
  use Anthropic's "Generate prompt" tool or iterate manually.

- **Not a runtime evaluator.** It does not run your prompt against a
  model, score the output, or check accuracy. For that, use
  [Promptfoo](https://promptfoo.dev), [Braintrust](https://braintrust.dev),
  or your own eval harness.

- **Not a runtime firewall.** It does not detect attacks at request
  time. For that, use [Lakera Guard](https://lakera.ai) or
  [Rebuff](https://github.com/protectai/rebuff).

- **Not for chat-window prompting.** When you're typing into Claude
  Code, ChatGPT, or Cursor, your prompts are ephemeral — they don't
  live in a file. There's nothing for a static analyzer to check. By
  the time `aiproof` could flag a contradiction, you've already gotten
  an answer.

> **Rule of thumb:** if your prompts live in a `git commit`, `aiproof`
> helps. If they live in a chat window, it doesn't.

## Tested against real-world projects

`aiproof` runs cleanly against a corpus of 20 popular open-source AI
projects pinned at exact SHAs (langchain, anthropic-cookbook,
openai-cookbook, llama-index, autogen, crewAI, dspy, haystack, marvin,
guidance, promptflow, instructor, mirascope, agno, llmware, semantic-kernel,
prompty, AutoGPT, babyagi, chatgpt-api). The corpus runs in CI as a
regression gate — see [`fixtures/corpus/CORPUS_REPORT.md`](fixtures/corpus/CORPUS_REPORT.md)
for the per-repo diagnostic counts and FP analysis.

**Honest scorecard for v0.1.4 against that corpus:**

- AIP006 (hardcoded credentials): **0 real findings.** An earlier release
  produced 2 hits that turned out to be docstring placeholders
  (`sk-ant-api03-xxxxxx...` and `"sk-randomAPIkey..."`). v0.1.4 added
  placeholder-suppression to AIP006 and those are now correctly skipped.
  The fact that 20 popular AI repos ship zero live keys is a credit to
  those maintainers, not a marketing claim for aiproof.
- AIP008 (jailbreak patterns): hits in test fixtures (intentionally
  embedded for adversarial-simulator testing) — auto-suppressed via
  fixture-path detection.
- AIP010, AIP015 (markdown noise): suppressed by the `is_prompt_shaped()`
  gate so README/CHANGELOG markdown isn't linted as prompts.

The 20-repo corpus matters more as a **false-positive regression gate**
than as a "look at all the bugs" demo: every change to a rule re-runs
against these baselines, and any FP-rate increase fails CI.

## What the output looks like

```text
error[AIP006]: hardcoded anthropic credential in prompt text
  ┌─ docs/setup.md:160:26
  │
160 │ ANTHROPIC_API_KEY=sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234
  │                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  │
  = See: https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP006.md

warning[AIP001]: conflicting instruction: "explain your reasoning" contradicts "only output json" above
  ┌─ prompts/agent.prompt.md:2:19
  │
2 │ Only output JSON. Explain your reasoning before answering.
  │                   ^^^^^^^^^^^^^^^^^^^^^^
  │
  = See: https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP001.md

warning[AIP009]: variable content within first ~1024 tokens defeats prompt caching
  ┌─ prompts/agent.prompt.md:4:1
  │
4 │ Answer this question: {query}
  │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  │
  = See: https://github.com/Frostbyte-Devs/aiproof/blob/main/docs/rules/AIP009.md
```

## Install

### Python (recommended for most users)

```bash
pip install aiproof
```

Wheels are published for **CPython 3.9+** on macOS (Intel + Apple Silicon),
Linux (x86_64 + aarch64), and Windows x86_64. Single wheel works across
all minor versions (abi3).

### Rust binary

```bash
cargo install aiproof-cli
```

### Prebuilt binaries

Download from [GitHub Releases](https://github.com/Frostbyte-Devs/aiproof/releases) — static binaries for macOS, Linux, Windows.

### From source

```bash
git clone https://github.com/Frostbyte-Devs/aiproof.git
cd aiproof
cargo install --path crates/aiproof-cli
```

## Quick start

**1. Lint a repo:**
```bash
aiproof .
```

You'll see a list of findings in the terminal, with line numbers and
squiggles. Exit code is `0` (clean), `1` (warnings), or `2` (errors).

**2. Fix what's auto-fixable:**
```bash
aiproof --fix .
```

Redacts hardcoded credentials, wraps user interpolations in input
boundary tags, removes near-duplicate instructions, and deletes unused
template variables. `--fix` is idempotent — run it twice, same result.

**3. Learn more about a rule:**
```bash
aiproof --explain AIP006
```

**4. Set up CI:**
```bash
aiproof --init
```

Prints a starter `.aiproofrc` and pre-commit hook snippet ready to paste.

**5. Integrate with GitHub Code Scanning:**
```yaml
# .github/workflows/prompts.yml
on: { push: { branches: [main] }, pull_request: }
jobs:
  aiproof:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pip install aiproof
      - run: aiproof --format sarif . > aiproof.sarif
      - uses: github/codeql-action/upload-sarif@v3
        with: { sarif_file: aiproof.sarif }
```

## Rules

`aiproof` ships 20 curated rules across 6 categories. Codes are stable;
`docs/rules/AIPxxx.md` contains the full explanation for each.

### Clarity — does the prompt say what you think it says?

| Code | Name | Severity |
|---|---|---|
| [AIP001](docs/rules/AIP001.md) | `conflicting-instructions` | warning |
| [AIP002](docs/rules/AIP002.md) | `ambiguous-output-format` | warning |
| [AIP003](docs/rules/AIP003.md) | `undefined-role` | info |
| [AIP004](docs/rules/AIP004.md) | `contradictory-tone` | warning |

### Security — is the prompt safe?

| Code | Name | Severity | Autofix |
|---|---|---|---|
| [AIP005](docs/rules/AIP005.md) | `unescaped-user-input` | warning | — |
| [AIP006](docs/rules/AIP006.md) | `hardcoded-credential` | **error** | ✅ redact |
| [AIP007](docs/rules/AIP007.md) | `missing-input-boundaries` | warning | ✅ wrap |
| [AIP008](docs/rules/AIP008.md) | `known-jailbreak-pattern` (26 signatures) | **error** | — |

### Efficiency — is the prompt cheap?

| Code | Name | Severity | Autofix |
|---|---|---|---|
| [AIP009](docs/rules/AIP009.md) | `cache-unfriendly-structure` (Anthropic prompt caching) | warning | — |
| [AIP010](docs/rules/AIP010.md) | `redundant-instruction` | info | ✅ remove |
| [AIP011](docs/rules/AIP011.md) | `excessive-tokens` | warning | — |
| [AIP012](docs/rules/AIP012.md) | `unused-template-variable` | info | ✅ remove |

### Behavior — will the prompt do what you want?

| Code | Name | Severity |
|---|---|---|
| [AIP013](docs/rules/AIP013.md) | `missing-format-example` | info |
| [AIP014](docs/rules/AIP014.md) | `undefined-tool-reference` | warning |
| [AIP015](docs/rules/AIP015.md) | `unhandled-placeholder` (TODO/FIXME/XXX) | warning |

### Portability — will the prompt work on other models?

| Code | Name | Severity |
|---|---|---|
| [AIP016](docs/rules/AIP016.md) | `claude-specific-tags-on-gpt` (`<thinking>` on GPT targets) | warning |
| [AIP017](docs/rules/AIP017.md) | `system-message-mismatch` (Anthropic shape on Gemini) | info |
| [AIP018](docs/rules/AIP018.md) | `temperature-determinism-mismatch` | warning |

### Best practice — is the prompt well-structured?

| Code | Name | Severity |
|---|---|---|
| [AIP019](docs/rules/AIP019.md) | `missing-few-shot-for-reasoning` | info |
| [AIP020](docs/rules/AIP020.md) | `system-message-overloaded` (>1500 tokens or >8 imperatives) | info |

## What `aiproof` reads

`aiproof` parses each file type with a format-aware parser and doesn't
lint arbitrary Markdown — rules that care about prompt semantics gate on
an **"is this actually a prompt?"** signal (explicit frontmatter role,
SDK call-site extraction, or a `.prompt.md` / `.prompt` extension).

| Input | Parser | Notes |
|---|---|---|
| `*.md`, `*.prompt.md` | `tree-sitter-md` | YAML frontmatter → `Role` |
| `*.j2`, `*.jinja`, `*.jinja2` | hand-rolled (`logos`) | variable table for AIP012 |
| `*.mustache` | hand-rolled (`logos`) | variable table for AIP012 |
| `*.yaml`, `*.yml` | Prompty-aware | `---`-fenced frontmatter → prompt body |
| `*.json` | MCP-aware | extracts `description` fields recursively |
| `*.py` | `tree-sitter-python` | extracts prompts from `messages.create`, `PromptTemplate`, `ChatPromptTemplate.from_messages`, `Agent(system=...)` |
| `*.ts`, `*.tsx` | `tree-sitter-typescript` | same call sites, template literals → `{0}`/`{1}` placeholders |
| `*.prompt` | raw | |

For Python and TypeScript, `aiproof` walks the AST and extracts the
string arguments passed to known LLM SDK call sites — so it finds the
real prompt your code ships, not a stringly guess.

## Configuration

`aiproof` looks for configuration in this precedence order (first match
wins):

1. CLI flags (`--select`, `--ignore`, `--target-model`)
2. `.aiproofrc` in the nearest ancestor directory (TOML)
3. `[tool.aiproof]` table in `pyproject.toml`
4. Built-in defaults

### `.aiproofrc` example

```toml
# Which files to lint. If omitted, aiproof picks up `.prompt.md`,
# `.j2`/`.jinja`/`.jinja2`, `.mustache`, files under `prompts/`,
# `templates/`, `system_prompts/`, plus SDK extraction for `.py`/`.ts`.
include = ["prompts/**/*.md", "src/**/*.py"]

# Common FP sources — safe to exclude by default.
exclude = [
    "docs/plans/**",
    "releasenotes/**",
    "tests/cassettes/**",
    "tests/recordings/**",
    "tests/fixtures/**",
    "node_modules/**",
    "target/**",
    ".venv/**",
]

# Selectively disable rules (supports `AIP*` wildcard).
# ignore = ["AIP019"]

# Target models enable portability rules (AIP016, AIP017, AIP018).
target_models = ["claude-4.7-opus", "gpt-4"]

# Approximate token budget used by AIP011.
max_tokens_budget = 4000
```

### All configuration keys

| Key | Type | Default | What it does |
|---|---|---|---|
| `include` | `Vec<String>` | auto-discover | Glob patterns. If set, files must match one. |
| `exclude` | `Vec<String>` | `[]` | Glob patterns. Always applied. |
| `select` | `Vec<String>` | `[]` (all enabled) | Enable-list. Supports `AIP*`. |
| `ignore` | `Vec<String>` | `[]` | Disable-list. Supports `AIP*`. |
| `target_models` | `Vec<String>` | `[]` | Enables portability rules when set. |
| `max_tokens_budget` | `Option<usize>` | `4000` | Token ceiling for AIP011. |
| `fix` | `bool` | `false` | Apply safe autofixes. |
| `unsafe_fixes` | `bool` | `false` | Also apply rule-declared unsafe fixes. |

## Python API

```python
import aiproof

# Lint a prompt string directly.
diagnostics = aiproof.check(
    source=open("prompts/agent.prompt.md").read(),
    path="prompts/agent.prompt.md",
    target_models=["claude-4.7-opus"],  # optional
    max_tokens_budget=4000,              # optional
)

for d in diagnostics:
    print(f"{d['severity'].upper()} {d['code']} "
          f"at line {d['start_line']}: {d['message']}")

# Exposes __version__ matching the installed wheel.
print(aiproof.__version__)  # "0.1.0"
```

Each diagnostic is a dict with these keys:

| Key | Type |
|---|---|
| `code` | `str` — e.g. `"AIP006"` |
| `message` | `str` — human-readable summary |
| `severity` | `"info"` / `"warning"` / `"error"` |
| `category` | `"clarity"` / `"security"` / `"efficiency"` / `"behavior"` / `"portability"` / `"best-practice"` |
| `start_line`, `start_col`, `end_line`, `end_col` | `int` — 1-based positions |
| `file` | `str` — the path you passed |

## CLI reference

```text
Static analyzer for AI prompts

Usage: aiproof [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Paths to scan. Defaults to current directory [default: .]

Options:
      --format <FORMAT>            Output format [pretty | json | sarif] [default: pretty]
      --select <CODE>              Enable specific rule codes (overrides config). Repeatable.
      --ignore <CODE>              Disable specific rule codes. Repeatable.
      --target-model <NAME>        Target model hint for portability rules. Repeatable.
      --fix                        Apply safe autofixes to files in place.
      --unsafe-fixes               Also apply autofixes the rule author marked unsafe.
      --color <COLOR>              [auto | always | never]
      --explain <CODE>             Print the bundled explanation for a rule code and exit 0.
      --init                       Print a starter config + pre-commit snippet and exit 0.
  -h, --help                       Print help
  -V, --version                    Print version
```

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Clean — no findings, or only `info` severity |
| `1` | One or more `warning` severity findings |
| `2` | One or more `error` severity findings, or invalid arguments / config |

Exit code is the max severity encountered in the run. Wire it into CI as
a hard gate on errors and a non-blocking reporter on warnings.

## FAQ

**Does `aiproof` call an LLM?**
No. Ever. Every check is pure text + AST analysis. There is no network
code in the binary; it works offline, in air-gapped environments, and on
restricted CI runners.

**How do I suppress a rule for a single file?**
Add a project-level ignore: `ignore = ["AIP019"]` in `.aiproofrc`, or
use `--ignore AIP019` on the command line. Per-line suppressions
(`# aiproof: ignore AIP019`) are on the v0.2 roadmap.

**How fast is it?**
Sub-50 ms per prompt file after warmup. A full scan of a 2800-file
langchain checkout runs in under 5 seconds. Criterion benchmarks live
under `crates/aiproof-cli/benches/`.

**Does `aiproof` touch my files?**
Only when you pass `--fix`. Writes are atomic (tempfile + rename) so a
crash mid-write leaves the original intact.

**What if a rule has too many false positives for me?**
File an issue with the repro, or disable locally via `ignore = ["AIPxxx"]`.
Every rule has a per-repo FP budget enforced against the 20-repo corpus
in CI — we take regressions seriously.

**Why Rust?**
Because `ruff` proved that a Rust-core linter with Python bindings wins
on speed (10-100× faster than pure Python) and distribution (single
static binary, no Python runtime required). Same playbook here.

**Will there be a VS Code extension / GitHub Action?**
Yes, both are on the v0.2 roadmap. Inline diagnostics on file save is
the obvious next step.

## How is this different from...

| Tool | What it does | Requires LLM calls? | Where it runs |
|---|---|---|---|
| **aiproof** | Static analysis of prompts | ❌ No | Editor / pre-commit / CI |
| Promptfoo | Runtime evaluation of prompt + output | ✅ Yes | CI / local eval runs |
| Braintrust | Runtime logging + evaluation | ✅ Yes | Production / CI |
| Lakera Guard | Runtime prompt-injection firewall | ✅ Yes | Production (API) |
| Rebuff | Runtime prompt-injection detection | ✅ Yes | Production |
| Guardrails.ai | Runtime structured-output validation | ✅ Yes | Production |
| PromptLayer | Runtime observability | ✅ Yes | Production |
| Prompty | Prompt YAML format spec | — | — |

`aiproof` is complementary to runtime tools — it catches issues *before*
they ever hit an LLM. Combine with Lakera for runtime + aiproof for
design-time coverage.

## Design principles

1. **No LLM calls.** Ever. If you find one, file a bug.
2. **Low false positives over high recall.** A disabled linter is a dead
   linter. Every rule is validated against 20 real AI projects with a
   ≤5 % FP budget.
3. **Beautiful output.** Line numbers, squiggles, color, context lines,
   and a `--explain` URL per finding. `ruff` proved this matters.
4. **Fast enough to run on save.** Sub-50 ms per prompt file.
5. **One-command install.** `pip install aiproof` or `cargo install aiproof-cli`.
6. **Deterministic output.** Same input + same config = byte-for-byte
   identical output. Required for CI diffing.

## Architecture

```text
crates/
├── aiproof-core    — Document, Rule trait, Diagnostic, Severity, Span
├── aiproof-parse   — per-format parsers + SDK call-site extractor
├── aiproof-rules   — the 20 bundled rules
├── aiproof-config  — .aiproofrc + pyproject.toml loader
├── aiproof-report  — pretty (codespan-reporting) / JSON / SARIF renderers
├── aiproof-cli     — clap CLI + file discovery + orchestration
└── aiproof-py      — pyo3 + maturin Python wheel
```

Each rule is a single file implementing the `Rule` trait with a pure
`check(&Document, &Ctx) -> Vec<Diagnostic>` function. Autofixes return
an `Option<Fix>` with `safe: bool`.

## Contributing

Contributions are welcome, especially new rules and FP reports from
real-world prompts.

### Dev setup

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

# Run against a target directory
cargo run --release -p aiproof-cli -- --format pretty <path>
```

### Adding a rule

1. Create `crates/aiproof-rules/src/rules/aipXXX_your_rule.rs` following
   the shape of an existing rule (e.g., `aip006_hardcoded_credential.rs`).
2. Register in `crates/aiproof-rules/src/rules/mod.rs`.
3. Add a test file at `crates/aiproof-rules/tests/aipXXX_your_rule.rs`
   with at least one positive and one negative case.
4. Write `docs/rules/AIPXXX.md` (under 100 words — what, why, example, fix).
5. Open a PR. CI will run the full corpus regression against all 20 repos.

Rules must meet a ≤5 % false-positive budget on the corpus to merge.

### Corpus regression

```bash
./scripts/sync_corpus.sh        # shallow-clone 20 AI repos at pinned SHAs
./scripts/generate_baselines.sh # run aiproof against each, save JSON baselines
```

Baselines live in `fixtures/corpus/*.baseline.json` and are diffed in CI.

## Roadmap

- **v0.2** — VS Code extension with inline diagnostics, GitHub Action
  (pre-made action YAML), per-line suppression comments, custom rule
  config DSL.
- **v0.3** — npm bindings (`@frostbyte/aiproof`), MCP server wrapping
  aiproof, additional SDK detection patterns (Cohere, Replicate,
  together.ai, Mistral), more parsers (Chezmoi templates, jsonnet).
- **v1.0** — embeddings-powered semantic rules, rule authoring SDK,
  hosted dashboard for team-wide FP dashboards.

## License

Apache-2.0. © 2026 Kristian Baer / [Northtek](https://github.com/Frostbyte-Devs).
