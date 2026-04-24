# Corpus baseline report — v0.1.0

Generated: 2026-04-23
Binary: `aiproof 0.1.0` (commit `d4eb1a6`)
Corpus: 20 open-source AI projects at `HEAD` (pinning SHAs is a v0.1.1 follow-up).

## Summary

| Metric | Value |
|---|---|
| Repos scanned | 20 |
| Total diagnostics | 1,463 |
| Errors | 45 |
| Warnings | 1,155 |
| Info | 263 |

Prior to the AIP010/AIP015 over-firing fix, the same scan produced
37,118 diagnostics — **96% of those were false positives on README-style
markdown**. The v0.1.0 release gates both rules behind `is_prompt_shaped()`.

## Per-repo counts

| Repo | Diagnostics |
|---|---:|
| haystack | 375 |
| promptflow | 273 |
| crewAI | 246 |
| llama_index | 204 |
| instructor | 88 |
| mirascope | 50 |
| agno | 35 |
| langchain | 35 |
| anthropic-cookbook | 33 |
| semantic-kernel | 33 |
| AutoGPT | 32 |
| autogen | 24 |
| llmware | 22 |
| prompty | 7 |
| dspy | 3 |
| chatgpt-api | 2 |
| marvin | 1 |
| babyagi | 0 |
| guidance | 0 |
| openai-cookbook | 0 |

## The launch story — real errors

### Hardcoded credentials in production documentation

Two credentials found in shipping documentation — both are the exact use case
aiproof exists for:

1. **AutoGPT** — `docs/content/classic/setup/index.md:160`
   - Hardcoded Anthropic credential.
   - Shipped in the setup guide. If the key is live, it's leaked.

2. **haystack** — `releasenotes/notes/secret-handling-for-components-d576a28135a224db.yaml:35`
   - Hardcoded OpenAI credential.
   - Ironic — it's in a release note about secret handling.

Both of these should become GitHub issues with the findings and a 90-day
disclosure window, filed by the maintainer of aiproof (Kristian). That's
the credibility play.

### Jailbreak-pattern detections (43)

All in test cassettes and adversarial-simulator fixtures:
- `crewAI`: 6 in VCR cassettes under `lib/crewai/tests/cassettes/agents/`
- `langchain`: 2 in `libs/langchain/tests/unit_tests/examples/test_specs/`
- `agno`: 2 in `cookbook/00_quickstart/`
- `promptflow`: 33 in `src/promptflow-evals/tests/recordings/azure/test_adv_simulator/`

These are debatable false positives. The tests *exist* to exercise
adversarial-simulator code paths, so the patterns are intentionally embedded.
A v0.1.1 config flag like `exclude_test_fixtures = true` (default off, default
exclude patterns include `**/tests/cassettes/**`, `**/tests/recordings/**`,
`**/fixtures/**`) would suppress these while keeping real findings.

## Rule fire frequencies

| Rule | Count | Notes |
|---|---:|---|
| AIP011 | 499 | excessive-tokens — mostly long planning/design docs |
| AIP004 | 300 | contradictory-tone — "concise" and "detailed" appear together in tutorials |
| AIP002 | 295 | ambiguous-output-format — "return JSON" without example in docs |
| AIP003 | 199 | undefined-role — multiple "you are a..." in explainers |
| AIP013 | 47 | missing-format-example |
| AIP008 | 43 | jailbreak-pattern (see above) |
| AIP019 | 17 | missing-few-shot-for-reasoning |
| AIP005 | 17 | unescaped-user-input |
| AIP009 | 17 | cache-unfriendly-structure |
| AIP007 | 16 | missing-input-boundaries |
| AIP001 | 11 | conflicting-instructions |
| AIP006 | 2 | hardcoded-credential |

## Observed FP patterns (fodder for v0.1.x)

1. **AIP011 over-fires on design docs.** docs/plans/ and release notes are
   linted as prompts because their .md extension hits discovery tier 2.
   Short-term: users should add `exclude = ["docs/plans/**", "releasenotes/**"]`.
   Longer-term: extend `is_prompt_shaped()` to deprioritize very-long docs
   without prompt-shaped signals.

2. **AIP003/AIP004 over-fire on tutorials.** Documentation that teaches
   prompt engineering references tone/role adjectives as examples, not as
   instructions. Consider adding "you should use X" and "instead of Y"
   heuristics that suppress pedagogical usage.

3. **AIP008 fires on intentional test fixtures.** Adding a
   `fixtures`/`cassettes`/`recordings` auto-exclude for the jailbreak rule
   specifically would cut 30+ noise diagnostics at zero cost to real
   detection.

4. **AIP002 fires on README code-fence examples.** When a README has
   `\`\`\`json\n{...}\n\`\`\`` followed by prose that says "return JSON", the
   current logic misses the fenced block as an example and flags anyway.
   Fix: treat any prior fenced block within the same document as satisfying
   the example requirement.

## Recommendation

- **Ship v0.1.0** as-is. The 2 real credential leaks + 4 safe autofixes
  are a legitimate launch story. The remaining noise is within an
  acceptable range for a first release (avg 73 diagnostics per repo,
  decreasing to ~30 per repo with `exclude = ["docs/plans/**"]`).
- **File v0.1.1 follow-ups** for each FP pattern above, targeting a
  corpus-wide < 500 total diagnostics, < 2% FP rate.
- **Pin real SHAs** in `CORPUS.toml` before tagging `v0.1.0` so
  regressions are measurable.
