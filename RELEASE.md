# Release checklist — v0.1.0

This doc is the hand-off from local development to a real public release.
It's intentionally terse. Run each step only when the one above is green.

## 0. Pre-flight (local)

- [x] `cargo test --workspace` — 173 tests pass
- [x] `cargo clippy --workspace --all-targets -- -D warnings` — clean
- [x] `cargo fmt --all -- --check` — clean
- [x] Python wheel builds via `maturin build --release` in `crates/aiproof-py/`
- [x] Binary smoke-test against synthetic + real projects
- [ ] **Corpus baselines generated and reviewed** (run `./scripts/generate_baselines.sh`, eyeball `fixtures/corpus/*.baseline.json`)
- [ ] Pin real SHAs in `fixtures/corpus/CORPUS.toml` (replace `HEAD` placeholders)

## 1. GitHub setup (one-time)

- [ ] Create the public repo:
  ```sh
  gh repo create Frostbyte-Devs/aiproof --public --description "ESLint for AI prompts" --homepage "https://aiproof.dev"
  ```
  (Pick the org is `Frostbyte-Devs`. The `repository` field in
  `Cargo.toml` is currently `https://github.com/Frostbyte-Devs/aiproof` — adjust
  if different.)

- [ ] Push `feat/v0` and `main`:
  ```sh
  git remote add origin git@github.com:Frostbyte-Devs/aiproof.git
  git push -u origin feat/v0
  # merge feat/v0 → main via PR, then:
  git push origin main
  ```

- [ ] Set branch protection on `main`: require CI to pass, require PR review
  (optional for solo but recommended).

## 2. Repository secrets

The release pipeline needs two tokens stored as GitHub Actions secrets.

- [ ] `PYPI_TOKEN` — scoped upload token for the `aiproof` project on PyPI.
  Create at https://pypi.org/manage/account/token/. After creating the PyPI
  project (first wheel upload is a one-time manual `twine upload` to claim
  the name), scope the token to the project.

- [ ] `CARGO_REGISTRY_TOKEN` — API token from https://crates.io/me.

Add both under **Settings → Secrets and variables → Actions** on the
GitHub repo.

## 3. First publish (manual, to claim names)

Publishing to PyPI and crates.io requires claiming the package names
before CI can auto-publish. Do these ONCE, then CI handles every future
release.

**PyPI:**
```sh
cd crates/aiproof-py
maturin build --release --strip
pip install twine
twine upload target/wheels/*.whl
```

**crates.io — publish in dependency order:**
```sh
cargo publish -p aiproof-core
sleep 10
cargo publish -p aiproof-parse
sleep 10
cargo publish -p aiproof-rules
sleep 10
cargo publish -p aiproof-config
sleep 10
cargo publish -p aiproof-report
sleep 10
cargo publish -p aiproof-cli
```

## 4. Release candidate

- [ ] Tag an RC to smoke-test the pipeline without committing to the
  real version:
  ```sh
  git tag v0.1.0-rc.1
  git push origin v0.1.0-rc.1
  ```

- [ ] Watch the `release` workflow under Actions. Check:
  - All 4 wheel matrix jobs pass
  - All 4 binary matrix jobs pass
  - Draft GitHub Release contains all artifacts
  - PyPI upload succeeds (or is a no-op if you pre-claimed with rc.1)
  - crates.io publish succeeds for all 6 crates

- [ ] If anything fails, fix, push a fix commit, tag `v0.1.0-rc.2`, repeat.

## 5. Real release

- [ ] `git tag v0.1.0 && git push origin v0.1.0`
- [ ] Verify `pip install aiproof` works from a fresh venv
- [ ] Verify `cargo install aiproof-cli` works from a fresh cargo home
- [ ] Verify prebuilt binaries run on macOS, Linux, Windows

## 6. Launch

- [ ] HN Show post with the "aiproof found N issues in langchain/anthropic-cookbook/..." screenshot (pull numbers from `fixtures/corpus/*.baseline.json`)
- [ ] X thread — same angle, screenshot of the pretty terminal output
- [ ] Blog post at https://aiproof.dev (if the domain is yours)
- [ ] File issues in receptive repos with the specific findings for
  validation of impact

## Post-launch

- Monitor FP reports from real users. Update `fp_budget.toml` per rule
  if you're getting false-positive noise.
- Tag `v0.1.z` patch releases aggressively — the faster you iterate on
  FP, the more trust you build.
- Start on v0.2: VS Code extension, GitHub Action, custom rule DSL.

## Rollback plan

- Yank bad crates: `cargo yank --version 0.1.z -p aiproof-<crate>`
- Delete bad PyPI upload: **not possible** — pre-test on TestPyPI if you're
  nervous. `pip install --index-url https://test.pypi.org/simple/ aiproof`.
- Delete bad GitHub Release: safe, just re-tag.

---

**Whatever you do, don't force-push `main` after publishing.** Version
numbers on crates.io and PyPI are permanent.
