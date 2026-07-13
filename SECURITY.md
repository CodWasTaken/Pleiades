# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.x   | ✅ |
| < 1.0   | ❌ (Development) |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in Pleiades, please follow these steps:

### Do Not

- Do **not** report security vulnerabilities through public GitHub issues
- Do **not** disclose the vulnerability publicly until it has been addressed

### Do

1. **Email** security concerns to the maintainers (see repository profile for contact)
2. **Provide** a detailed description of the vulnerability
3. **Include** steps to reproduce the issue
4. **Mention** the version and environment where the vulnerability was found

### What to Expect

- **Acknowledgment** within 48 hours
- **Assessment** and severity classification within 5 business days
- **Mitigation** timeline based on severity:
  - **Critical**: Patch within 7 days
  - **High**: Patch within 14 days
  - **Medium**: Patch within 30 days
  - **Low**: Next release cycle

## Security Best Practices

### API Keys
- Use environment variables (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.)
- Never commit API keys to version control
- Prefer environment-variable references such as `${OPENAI_API_KEY}` in configuration
- `config show` masks API keys unless the explicit `--raw` flag is used

### Plugin Security
- Plugin hooks run as local shell commands with the current user's permissions
- Review every plugin manifest and hook command before installation
- Only install plugins from trusted sources

### Telemetry
- Pleiades collects **no telemetry** by default
- Any telemetry is opt-in and clearly documented
- No data is sent without explicit user consent

## Security Architecture

See [ARCHITECTURE.md](docs/ARCHITECTURE.md#security-architecture) for detailed security design including:
- Permission levels (ReadOnly, WorkspaceWrite, Dangerous)
- Approval modes (Auto, Ask, Deny, Plan)
- Environment-variable secret interpolation and masked configuration output
- Tool permission checks and timeouts
