<!--
Thanks for contributing to aiproof. A few asks:

1. Title: use Conventional Commits (feat:, fix:, docs:, test:, ci:, chore:).
2. Fill in the sections below.
3. Make sure `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` are clean locally.
4. If you added a rule, run the corpus regression — see CONTRIBUTING.md.
-->

## What

<!-- One or two sentences: what does this PR change? -->

## Why

<!-- What problem does it solve? Link related issues with `Closes #NNN`. -->

## Type of change

- [ ] New rule
- [ ] Bug fix
- [ ] Documentation
- [ ] Refactor
- [ ] Performance improvement
- [ ] Test coverage
- [ ] CI / build
- [ ] Other (describe)

## Checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] Updated `CHANGELOG.md` under `[Unreleased]` if the change is user-visible
- [ ] Added / updated tests
- [ ] If this adds a new rule: `docs/rules/AIPXXX.md` written, corpus regression run

## Corpus regression (if touching rule behavior)

<!-- If the rule-engine behavior changed, paste the diff in aggregate diagnostic counts per repo. -->

```
Before: <N> total diagnostics
After:  <M> total diagnostics
```
