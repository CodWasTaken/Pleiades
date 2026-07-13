# Providers and models

Pleiades implements native OpenAI and Anthropic adapters plus an OpenAI-compatible adapter used by OpenRouter, Groq, DeepSeek, local servers, and other compatible endpoints.

Use `provider list`, `provider info NAME`, and `provider test NAME`. Model discovery and aliases are managed with `model discover`, `model list`, `model alias ALIAS MODEL`, and `model set-default MODEL`. Global `--provider` and `--model` flags override configuration for a command.
