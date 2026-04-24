# Changelog

All notable changes to aiproof are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [SemVer](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-04-24

First public release.

### Added

- **20 curated rules** across 6 categories (clarity, security, efficiency, behavior, portability, best-practice):
  - `AIP001` conflicting-instructions · `AIP002` ambiguous-output-format · `AIP003` undefined-role · `AIP004` contradictory-tone
  - `AIP005` unescaped-user-input · `AIP006` hardcoded-credential (autofix) · `AIP007` missing-input-boundaries (autofix) · `AIP008` known-jailbreak-pattern (26 signatures)
  - `AIP009` cache-unfriendly-structure · `AIP010` redundant-instruction (autofix) · `AIP011` excessive-tokens · `AIP012` unused-template-variable (autofix)
  - `AIP013` missing-format-example · `AIP014` undefined-tool-reference · `AIP015` unhandled-placeholder
  - `AIP016` claude-specific-tags-on-gpt · `AIP017` system-message-mismatch · `AIP018` temperature-determinism-mismatch
  - `AIP019` missing-few-shot-for-reasoning · `AIP020` system-message-overloaded
- **Parsers** for plain text, Markdown (tree-sitter-md), Jinja2, Mustache, YAML (Prompty-aware), JSON (MCP-aware), Python (tree-sitter-python), TypeScript (tree-sitter-typescript).
- **SDK call-site extraction** from `.py` and `.ts`/`.tsx`: Anthropic `messages.create`, OpenAI `chat.completions.create`, LangChain `PromptTemplate` and `ChatPromptTemplate.from_messages`, and common `Agent(system=...)` shapes.
- **Four safe autofixes**: AIP006 (redact credential), AIP007 (wrap user input in delimiter tags), AIP010 (remove duplicate sentence), AIP012 (remove unused template variable).
- **Three output formats**: `pretty` (codespan-reporting with colored squiggles), `json` (stable schema), `sarif` (SARIF 2.1.0 for GitHub Code Scanning).
- **CLI**: `aiproof .`, `--fix`, `--explain AIPxxx`, `--init`, `--format`, `--select`/`--ignore` (with `AIP*` wildcards), `--target-model`, `--color`.
- **Configuration cascade**: `.aiproofrc` (TOML) → `pyproject.toml [tool.aiproof]` → defaults. Unknown keys rejected.
- **Python wheel** via `pyo3` + `maturin` (abi3-py39, single wheel for Python 3.9+) with a `check()` API returning a list of diagnostic dicts.
- **Corpus regression harness**: `CORPUS.toml` pinning 20 real AI projects at exact SHAs, `scripts/sync_corpus.sh` / `generate_baselines.sh` to materialize and scan.
- **Release pipeline** (GitHub Actions): wheel matrix (Linux x86/aarch64, macOS universal2, Windows), binary matrix (same), PyPI + crates.io + GitHub Releases publish on tag.
- 173 tests across 7 crates. `cargo clippy -D warnings` clean.

### Corpus findings on release

Running against 20 real AI projects surfaced, among other findings:

- `AIP006` — hardcoded Anthropic credential in [`Significant-Gravitas/AutoGPT`](https://github.com/Significant-Gravitas/AutoGPT) `docs/content/classic/setup/index.md:160`
- `AIP006` — hardcoded OpenAI credential in [`deepset-ai/haystack`](https://github.com/deepset-ai/haystack) `releasenotes/notes/secret-handling-for-components-*.yaml:35`

See [`fixtures/corpus/CORPUS_REPORT.md`](fixtures/corpus/CORPUS_REPORT.md) for the full breakdown.

### Known limitations (v0.1.0)

- **`AIP014` undefined-tool-reference** ships as a v0 stub returning zero diagnostics — tool registry discovery comes in v0.2.
- **False-positive noise** on AIP011 / AIP003 / AIP004 in general documentation. Mitigate with `exclude = ["docs/plans/**", "releasenotes/**"]` in `.aiproofrc` (included in `--init` defaults).
- **Per-line rule suppression** (`# aiproof: ignore AIPxxx`) is not yet supported; v0.2.

[Unreleased]: https://github.com/Frostbyte-Devs/aiproof/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Frostbyte-Devs/aiproof/releases/tag/v0.1.0
