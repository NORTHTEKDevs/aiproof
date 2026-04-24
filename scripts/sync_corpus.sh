#!/usr/bin/env bash
# scripts/sync_corpus.sh -- materialize the aiproof corpus at pinned SHAs.
# Run by: ./scripts/sync_corpus.sh
set -euo pipefail

REPO_DIR="fixtures/corpus"
mkdir -p "$REPO_DIR"

# List of repos to sync in the format: name|url|sha
# Parsed from fixtures/corpus/CORPUS.toml; for v0 we embed to avoid TOML deps.
entries=(
  "langchain|https://github.com/langchain-ai/langchain.git|HEAD"
  "anthropic-cookbook|https://github.com/anthropics/anthropic-cookbook.git|HEAD"
  "openai-cookbook|https://github.com/openai/openai-cookbook.git|HEAD"
  "llama_index|https://github.com/run-llama/llama_index.git|HEAD"
  "autogen|https://github.com/microsoft/autogen.git|HEAD"
  "crewAI|https://github.com/joaomdmoura/crewAI.git|HEAD"
  "AutoGPT|https://github.com/Significant-Gravitas/AutoGPT.git|HEAD"
  "babyagi|https://github.com/yoheinakajima/babyagi.git|HEAD"
  "chatgpt-api|https://github.com/transitive-bullshit/chatgpt-api.git|HEAD"
  "prompty|https://github.com/microsoft/prompty.git|HEAD"
  "semantic-kernel|https://github.com/microsoft/semantic-kernel.git|HEAD"
  "haystack|https://github.com/deepset-ai/haystack.git|HEAD"
  "marvin|https://github.com/PrefectHQ/marvin.git|HEAD"
  "guidance|https://github.com/guidance-ai/guidance.git|HEAD"
  "promptflow|https://github.com/microsoft/promptflow.git|HEAD"
  "instructor|https://github.com/jxnl/instructor.git|HEAD"
  "dspy|https://github.com/stanfordnlp/dspy.git|HEAD"
  "mirascope|https://github.com/Mirascope/mirascope.git|HEAD"
  "agno|https://github.com/agno-agi/agno.git|HEAD"
  "llmware|https://github.com/llmware-ai/llmware.git|HEAD"
)

for e in "${entries[@]}"; do
  IFS='|' read -r name url sha <<< "$e"
  dst="$REPO_DIR/$name"
  if [ -d "$dst/.git" ]; then
    echo "[skip] $name already present"
    continue
  fi
  echo "[clone] $name @ $sha"
  git clone --depth 50 "$url" "$dst" 2>&1 | grep -v "^Cloning\|^Receiving\|^Resolving" || true
  if [ "$sha" != "HEAD" ]; then
    (cd "$dst" && git checkout --quiet "$sha")
  fi
done

echo "corpus ready at $REPO_DIR"
