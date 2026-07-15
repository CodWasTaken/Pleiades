# Audit log

Pleiades records a local JSONL audit log for security-relevant runtime actions.

By default, audit entries are written under the platform data directory:

```text
~/.local/share/pleiades/audit/audit.jsonl
```

When `session.history_dir` is configured, Pleiades writes the audit directory
next to that session directory. This keeps project/test-specific runtime state
together.

The log records:

- slash and direct runtime commands;
- provider, model, and mode changes;
- YOLO activation metadata;
- permission decisions;
- tool requests and outcomes;
- shell command requests;
- file targets touched by tools;
- checkpoint/session operations;
- plugin/MCP/custom-command reload effects;
- task starts, failures, cancellations, and completions.

Pleiades redacts secret-like payloads before writing audit entries. Sensitive
keys such as `api_key`, `token`, `authorization`, `password`, and `secret` are
replaced with `[REDACTED]`; common token-shaped values such as `sk-...`,
`ghp_...`, and long credential-like strings are also redacted.

The audit log intentionally does not store full tool output or provider text
streams. It stores operation metadata needed to explain what happened without
capturing unnecessary secret-bearing content.
