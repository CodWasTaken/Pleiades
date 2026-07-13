# Providers and models

Pleiades implements native OpenAI and Anthropic adapters plus an OpenAI-compatible adapter used by OpenRouter, Groq, DeepSeek, local servers, and other compatible endpoints.

## OpenAI authentication

Run `pleiades setup` to choose one of two supported modes:

- **ChatGPT subscription**: `pleiades auth login` delegates the browser or device-code flow to the official Codex CLI. Pleiades invokes `codex exec` in an ephemeral, read-only mode and does not read, copy, or refresh the Codex credential cache. This provider is named `openai-subscription`.
- **Platform API key**: Pleiades calls the OpenAI API directly with `${OPENAI_API_KEY}`. API usage has separate usage-based billing and is not funded by a ChatGPT Plus, Pro, Business, or Enterprise subscription.

See OpenAI's official [Codex authentication guide](https://developers.openai.com/codex/auth/) and [ChatGPT versus Platform billing explanation](https://help.openai.com/en/articles/9039756-billing-settings-in-chatgpt-vs-platform).

Useful commands:

```console
pleiades auth login
pleiades auth login --device
pleiades auth status
pleiades provider test openai-subscription
pleiades doctor
```

Subscription mode intentionally does not import OAuth tokens into Pleiades. It currently returns Codex's final response rather than token-by-token streaming and delegates model selection to Codex when `codex-default` is configured.

Use `provider list`, `provider info NAME`, and `provider test NAME`. Model discovery and aliases are managed with `model discover`, `model list`, `model alias ALIAS MODEL`, and `model set-default MODEL`. Global `--provider` and `--model` flags override configuration for a command.
