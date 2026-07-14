# Providers and models

Pleiades implements native OpenAI and Anthropic adapters plus an OpenAI-compatible adapter used by OpenRouter, Groq, DeepSeek, local servers, and other compatible endpoints.

## OpenAI authentication

Run `pleiades setup` to choose one of two supported modes:

- **ChatGPT subscription**: `pleiades auth login` delegates the browser or device-code flow to the official Codex CLI. Pleiades invokes `codex exec` ephemerally and does not read, copy, or refresh the Codex credential cache. This provider is named `openai-subscription`.
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

Subscription mode intentionally does not import OAuth tokens into Pleiades. Codex performs autonomous agent work in the launch directory and streams command and file activity back to the Pleiades interface. The default `workspace-write` sandbox permits writes only within that workspace; use `/mode plan` for read-only work. Model selection remains delegated to Codex when `codex-default` is configured.

Use `provider list`, `provider info NAME`, and `provider test NAME`. Model discovery and aliases are managed with `model discover`, `model list`, `model alias ALIAS MODEL`, and `model set-default MODEL`. Global `--provider` and `--model` flags override configuration for a command.

Inside the live workspace, `/provider list`, `/provider info NAME`,
`/provider use NAME`, `/provider remove NAME`, and `/provider reload` use the
shared provider service. `/provider add` opens the provider-wizard flow. The
registry generates help and nested completion for these commands, and provider
documents never contain resolved credential values.

`/provider test NAME [MODEL]` and `pleiades provider test NAME` now execute
the same service operation and only report success after the provider stream
emits a completion event. Model discovery and mutations are also available in
the workspace through `/model list [PROVIDER] [SEARCH]`, `/model info NAME`,
`/model discover`, `/model use NAME`, `/model alias ALIAS MODEL`, and
`/model unalias ALIAS`. Use `/model favorite NAME` to toggle a favorite,
`/model favorites` to inspect preferences, and `/model reasoning LEVEL` to
select `minimal`, `low`, `medium`, or `high` reasoning effort for adapters that
support it. The same operations are available as `pleiades model favorite`,
`pleiades model favorites`, and `pleiades model reasoning`. Model configuration
writes preserve environment variable references instead of persisting resolved
credentials.
