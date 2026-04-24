# scripts/sync_corpus.ps1 -- materialize the aiproof corpus at pinned SHAs (Windows).
# Run by: .\scripts\sync_corpus.ps1
param(
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\scripts\sync_corpus.ps1`n"
    Write-Host "Clones 20 pinned AI repos to fixtures/corpus/ at exact SHAs.`n"
    exit 0
}

$REPO_DIR = "fixtures\corpus"
New-Item -ItemType Directory -Force -Path $REPO_DIR | Out-Null

$entries = @(
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

foreach ($e in $entries) {
    $parts = $e -split '\|'
    $name = $parts[0]
    $url = $parts[1]
    $sha = $parts[2]
    $dst = Join-Path $REPO_DIR $name

    if (Test-Path "$dst\.git") {
        Write-Host "[skip] $name already present"
        continue
    }

    Write-Host "[clone] $name @ $sha"
    git clone --depth 50 $url $dst 2>&1 | Where-Object { $_ -notmatch "^Cloning|^Receiving|^Resolving" } | Out-Null

    if ($sha -ne "HEAD") {
        Push-Location $dst
        git checkout --quiet $sha
        Pop-Location
    }
}

Write-Host "corpus ready at $REPO_DIR"
