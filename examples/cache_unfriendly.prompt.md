---
role: system
---
You are helping {user_name}. Use the following guidance to answer their question.

# Guidance

This is a long stable section of guidance that would normally be perfect for prompt caching on Anthropic. Unfortunately, because {user_name} appears before it, the cache prefix is broken and you pay full-token cost on every call instead of 0.1x.

Move `{user_name}` to the end of the system prompt (or the first user message) to fix this.
