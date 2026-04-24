# aiproof corpus

Pinned snapshots of 20 real-world open-source AI projects. Used for:

- **Regression detection**: change in diagnostic output between releases
- **False-positive budget enforcement**: per-rule FP ceilings across diverse code patterns

## Usage

```sh
./scripts/sync_corpus.sh        # shallow-clone all repos at pinned SHAs
./scripts/generate_baselines.sh # run aiproof and save JSON baselines
```

## Repository manifest

See `CORPUS.toml` for the pinned list of repos, URLs, and commit SHAs. Each `[[repo]]` entry defines a snapshot. Run `sync_corpus.sh` to materialize them locally.

## Not tracked in git

The cloned repos themselves live in `fixtures/corpus/<name>/` and are `.gitignore`d to avoid bloating the repo. Only:

- **Manifest** (`CORPUS.toml`) — the pins
- **Baselines** (`fixtures/corpus/*.baseline.json`) — regression snapshots

are checked into version control.

## False-positive budget

Each diagnostic rule declares `FP_BUDGET` in its module (typical: 5%, one flag per ~20 prompts). CI parses the baseline JSON files and counts diagnostics per rule per repo:

- If any rule exceeds its budget in the corpus, CI fails the release gate.
- Budgets are intentionally conservative for v0 (expect ~2-3 FPs per rule across 20 repos).
- After each release, budgets can be adjusted based on real-world patterns.

## Corpus composition

The 20 repos span:

- **Framework** — LangChain, LLaMA Index, Semantic Kernel, Haystack
- **Agent orchestration** — AutoGen, CrewAI, AutoGPT, BabyAGI
- **Prompt engineering** — OpenAI Cookbook, Anthropic Cookbook, dspy, DSPy
- **Developer tooling** — Prompty, PromptFlow, Instructor, Marvin, Mirascope, Guidance
- **Emerging frameworks** — Agno, LLMware

All repos have active maintenance, diverse code patterns (Python, TypeScript, YAML, Markdown), and realistic prompt/system-message usage.
