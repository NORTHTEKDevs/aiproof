# aiproof-cli

The `aiproof` binary — a static analyzer for AI prompts ("ESLint for prompts").

## Install

```sh
cargo install aiproof-cli
```

## Usage

```sh
aiproof .                 # lint the current directory
aiproof --fix .           # apply safe autofixes
aiproof --explain AIP006  # read a rule's docs
aiproof --init            # print a starter .aiproofrc + pre-commit hook
```

Runs against `*.md`, `*.prompt.md`, `*.j2`, `*.mustache`, `*.yaml`,
`*.json` (MCP schemas), `*.py` (extracts prompts from Anthropic / OpenAI /
LangChain SDK calls), and `*.ts`/`*.tsx`.

Three output formats: `pretty` (default), `json`, `sarif` (for GitHub Code Scanning).

**Zero LLM calls, zero network, zero inference cost.** All checks are
pure text + AST analysis.

See the full documentation at <https://github.com/Frostbyte-Devs/aiproof>.

License: Apache-2.0.
