"""Demonstrates prompts extracted from real SDK call sites.

aiproof walks the Python AST, finds known LLM SDK calls, and extracts the
string arguments as Document values. Rules then run against those extracted
strings as if they were standalone prompt files.
"""

from anthropic import Anthropic

client = Anthropic()


def ask_claude(query: str, name: str) -> str:
    response = client.messages.create(
        model="claude-4.7-opus",
        system="Always return exactly this JSON schema. Do not explain.",
        messages=[
            {"role": "user", "content": f"Hello {name}, here is the query: {query}"},
        ],
        temperature=0.9,  # AIP018: deterministic prompt + high temperature is a mismatch
    )
    return response.content[0].text


# LangChain shape — also detected:
from langchain.prompts import PromptTemplate

summarize = PromptTemplate.from_template(
    "Summarize this document: {document}. Output JSON."
)
