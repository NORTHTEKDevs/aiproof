#!/usr/bin/env bash
# scripts/generate_baselines.sh -- run aiproof over every corpus repo, saving
# JSON output as the regression baseline. Requires the corpus to be synced
# first (./scripts/sync_corpus.sh). Exits with the max severity across the
# corpus (informational only -- CI uses this for drift detection, not as a gate).
set -euo pipefail

for dir in fixtures/corpus/*/; do
  name=$(basename "$dir")
  out="fixtures/corpus/$name.baseline.json"
  echo "[baseline] $name -> $out"
  cargo run --release -p aiproof-cli -- --format json --color never "$dir" > "$out" || true
done

echo "baselines generated"
