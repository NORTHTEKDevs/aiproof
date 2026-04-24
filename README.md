# aiproof

> ESLint for AI prompts.

`aiproof` is a static analyzer for the prompts you feed to LLMs. It reads
prompt files and LLM-SDK call sites directly from your repo and reports
issues across six categories — clarity, security, efficiency, behavior,
portability, and best-practice — with **zero LLM calls, zero network, zero
inference cost**.

```text
error[AIP006]: hardcoded anthropic credential in prompt text
  ┌─ prompt.md:3:26
  │
3 │ Your internal API key is sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234
  │                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  │
  = See: https://aiproof.dev/rules/AIP006
```

## Install

```bash
# Python wheel (recommended)
pip install aiproof

# Rust binary
cargo install aiproof-cli

# Prebuilt binaries
# Download from https://github.com/northtek/aiproof/releases
```

Python wheels are built for macOS, Linux, and Windows on CPython 3.9+.

## Quick start

```bash
# Lint the current directory
aiproof .

# Apply safe autofixes (credential redaction, input boundary wrapping, etc.)
aiproof --fix .

# Explain a rule
aiproof --explain AIP006

# Emit JSON for tooling
aiproof --format json .

# Emit SARIF for GitHub Code Scanning
aiproof --format sarif . > results.sarif

# Print a starter config + pre-commit hook snippet
aiproof --init
```

## What it finds

`aiproof` ships 20 rules across 6 categories. A few highlights:

| Code | Category | What it catches |
|---|---|---|
| AIP001 | clarity | Instructions that contradict each other |
| AIP006 | security | Hardcoded API keys / credentials in prompt text (**autofix**) |
| AIP007 | security | User-interpolated content without delimiter tags (**autofix**) |
| AIP008 | security | Known jailbreak / injection patterns |
| AIP009 | efficiency | Variable content in the first 1024 tokens defeats Anthropic prompt caching |
| AIP011 | efficiency | Prompts over the token budget |
| AIP016 | portability | `<thinking>` tags on a prompt targeting GPT |
| AIP018 | portability | Prompt asks for deterministic output while temperature > 0.3 |

See [`docs/rules/`](docs/rules) for the full list.

## What it reads

| Input | Parser |
|---|---|
| `*.md`, `*.prompt.md` | Markdown (tree-sitter-md) with YAML frontmatter |
| `*.j2`, `*.jinja`, `*.jinja2` | Jinja2 (hand-rolled) |
| `*.mustache` | Mustache (hand-rolled) |
| `*.yaml`, `*.yml` | Prompty-aware YAML |
| `*.json` | MCP tool schemas (description extraction) |
| `*.py` | Python AST — extracts prompts from Anthropic, OpenAI, LangChain SDK calls |
| `*.ts`, `*.tsx` | TypeScript AST — same for JS SDKs |

## Configuration

`aiproof` looks for configuration in either `.aiproofrc` (TOML) or
`pyproject.toml` under `[tool.aiproof]`. Example:

```toml
# .aiproofrc
include = ["prompts/**/*.md", "src/**/*.py"]
exclude = ["docs/plans/**", "tests/**"]
ignore  = ["AIP019"]
target_models = ["claude-4.7-opus", "gpt-4"]
max_tokens_budget = 4000
```

Target models enable the portability rules (AIP016–AIP018).

## Python API

```python
import aiproof

diags = aiproof.check(open("prompt.md").read(), "prompt.md")
for d in diags:
    print(f"{d['severity'].upper()} {d['code']} at line {d['start_line']}: {d['message']}")
```

## Exit codes

- `0` — clean (or only Info-severity findings)
- `1` — Warning severity found
- `2` — Error severity found, or invalid arguments / config

Wire it into CI:

```yaml
# .github/workflows/prompts.yml
- run: pip install aiproof && aiproof --format sarif . > aiproof.sarif
- uses: github/codeql-action/upload-sarif@v3
  with: { sarif_file: aiproof.sarif }
```

## Design principles

1. **No LLM calls, ever.** All checks are pure text + AST analysis.
2. **Low false positives over high recall.** A disabled linter is a dead
   linter. Every rule is validated against real open-source AI projects.
3. **Beautiful output.** Full line numbers, squiggles, color, context.
4. **Fast enough to run on save.** Sub-50ms per prompt file.
5. **Installable in one command.** `pip install aiproof` or `cargo install
   aiproof-cli`.

## Status

v0.1.0 — 20 rules, 173 tests, three output formats, four autofixes. See
[`docs/plans/2026-04-23-aiproof-design.md`](docs/plans/2026-04-23-aiproof-design.md)
for the full design, and the [roadmap](#roadmap) for what's next.

## Roadmap

- **v0.2** — VS Code extension, GitHub Action, custom rule config DSL.
- **v0.3** — npm bindings (JS/TS native), MCP server wrapping aiproof,
  more SDK detection patterns (Cohere, Replicate, together.ai).
- **v1.0** — embeddings-powered semantic rules, rule authoring SDK.

## Contributing

Rules live in [`crates/aiproof-rules/src/rules/`](crates/aiproof-rules/src/rules/).
Each rule is a single file implementing the `Rule` trait. Tests use
`insta` snapshots plus live fixtures. New rules are accepted via PR if
they meet a 5% false-positive budget on the corpus.

## License

Apache-2.0. © 2026 Kristian Baer / Northtek.
