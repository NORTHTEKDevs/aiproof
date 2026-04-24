# Examples

Sample prompts that demonstrate every aiproof rule. Each file contains
a deliberate issue so you can see what `aiproof` flags and why.

Run `aiproof examples/` from the repo root to see findings across all
samples.

## Files

| File | Demonstrates |
|---|---|
| [`conflicting_instructions.prompt.md`](conflicting_instructions.prompt.md) | `AIP001` — "output JSON" + "explain your reasoning" in the same prompt |
| [`hardcoded_credential.prompt.md`](hardcoded_credential.prompt.md) | `AIP006` — an Anthropic API key inline (autofix redacts it) |
| [`user_input_injection.prompt.md`](user_input_injection.prompt.md) | `AIP007` — `{user_query}` without delimiter tags (autofix wraps it) |
| [`jailbreak_pattern.prompt.md`](jailbreak_pattern.prompt.md) | `AIP008` — "ignore previous instructions" signature |
| [`cache_unfriendly.prompt.md`](cache_unfriendly.prompt.md) | `AIP009` — interpolation near the start of a system prompt |
| [`overloaded_system.prompt.md`](overloaded_system.prompt.md) | `AIP020` — a system message with 10+ imperatives |
| [`prompty_example.yaml`](prompty_example.yaml) | Prompty frontmatter YAML shape |
| [`sdk_call_site.py`](sdk_call_site.py) | Python Anthropic SDK call → extracted prompt |
| [`sdk_call_site.ts`](sdk_call_site.ts) | TypeScript OpenAI SDK call → extracted prompt |

## Autofix demo

```bash
# Copy the examples to a scratch dir first so you can see the redaction
cp -r examples/ /tmp/aiproof-demo
aiproof --fix /tmp/aiproof-demo

diff -r examples /tmp/aiproof-demo
```

Safe autofixes redact credentials, wrap user inputs, and remove
duplicates without changing prompt semantics.
