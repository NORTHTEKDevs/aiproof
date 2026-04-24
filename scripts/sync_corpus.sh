#!/usr/bin/env bash
# scripts/sync_corpus.sh — materialize the aiproof corpus at pinned SHAs.
# Source of truth: fixtures/corpus/CORPUS.toml. Keep both in sync.
# Run by: ./scripts/sync_corpus.sh
set -euo pipefail

REPO_DIR="fixtures/corpus"
mkdir -p "$REPO_DIR"

entries=(
  "langchain|https://github.com/langchain-ai/langchain.git|3dd0ad958eb2d5a51a4055e104598bb26aeb3b65"
  "anthropic-cookbook|https://github.com/anthropics/anthropic-cookbook.git|753ddfe35fdc7e310a45cadcfe314ba6809672f8"
  "openai-cookbook|https://github.com/openai/openai-cookbook.git|564a630c0bdfa56b18a87f0f63e17cd18af9f202"
  "llama_index|https://github.com/run-llama/llama_index.git|a3aeb31d24d79eca04635e5be17920f70aa8e9cf"
  "autogen|https://github.com/microsoft/autogen.git|027ecf0a379bcc1d09956d46d12d44a3ad9cee14"
  "crewAI|https://github.com/joaomdmoura/crewAI.git|b0e2fda105c2e0c05c7abb1f53800443ffd582ea"
  "AutoGPT|https://github.com/Significant-Gravitas/AutoGPT.git|cf6d7034fa3af819cbad758029764140e7eca94d"
  "babyagi|https://github.com/yoheinakajima/babyagi.git|fa8930ebe72a82e5ad57b356e7cbec96290e5bb2"
  "chatgpt-api|https://github.com/transitive-bullshit/chatgpt-api.git|beffa8ecfaffcf2e2435b077d5b7cd2dc33298bb"
  "prompty|https://github.com/microsoft/prompty.git|607c0579aedbc55ab89f3f22317404f76960c2e9"
  "semantic-kernel|https://github.com/microsoft/semantic-kernel.git|95b1bf85d8e72f70d457a81d2f179465616914db"
  "haystack|https://github.com/deepset-ai/haystack.git|602c4976ba6f494d2fd664582a37a770716429b8"
  "marvin|https://github.com/PrefectHQ/marvin.git|5c66f49708c2c445e6af3eb3eacf41251f0d645a"
  "guidance|https://github.com/guidance-ai/guidance.git|5413339aad8d36ce49df29902a7730d025bd6027"
  "promptflow|https://github.com/microsoft/promptflow.git|6bfdec06ef16d875ca3b1744a1ef133f08c35340"
  "instructor|https://github.com/jxnl/instructor.git|3f1d6ddb084b8a0da3eb0665051293d381383b41"
  "dspy|https://github.com/stanfordnlp/dspy.git|109568bf713121e09a5b9b58eba5a8903d9091b1"
  "mirascope|https://github.com/Mirascope/mirascope.git|3d0342a7815cbe6554ea3c90dd6c0036c4eef888"
  "agno|https://github.com/agno-agi/agno.git|b5b4dfd8de63197152e5a4003d6a76304c7d9236"
  "llmware|https://github.com/llmware-ai/llmware.git|556641378153f913f647725529e6f824f0dc8ba5"
)

for e in "${entries[@]}"; do
  IFS='|' read -r name url sha <<< "$e"
  dst="$REPO_DIR/$name"
  if [ -d "$dst/.git" ]; then
    current=$(cd "$dst" && git rev-parse HEAD 2>/dev/null || echo "")
    if [ "$current" = "$sha" ]; then
      echo "[skip] $name already at $sha"
      continue
    fi
    echo "[update] $name: $current -> $sha"
    (cd "$dst" && git fetch origin && git checkout --quiet "$sha") || \
      echo "[warn] failed to update $name to $sha"
    continue
  fi
  echo "[clone] $name @ $sha"
  git clone --depth 50 "$url" "$dst" 2>&1 | grep -v "^Cloning\|^Receiving\|^Resolving" || true
  if ! (cd "$dst" && git fetch --depth 50 origin "$sha" 2>/dev/null; git checkout --quiet "$sha") 2>/dev/null; then
    echo "[warn] could not checkout $sha for $name — left at default branch HEAD"
  fi
done

echo "corpus ready at $REPO_DIR"
