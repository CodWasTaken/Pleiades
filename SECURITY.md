# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 2.0.x   | ✅ |
| 1.x     | Security fixes only |
| < 1.0   | ❌ |

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
- Pleiades contains no product telemetry collection
- Provider requests necessarily send the selected conversation context to the configured provider

## Security Architecture

See [ARCHITECTURE.md](docs/ARCHITECTURE.md#tool-and-permission-boundary) for detailed security design including:
- Permission levels (ReadOnly, WorkspaceWrite, Dangerous)
- Plan, Agent, Auto, and YOLO modes plus once/session modal decisions
- Environment-variable secret interpolation and masked configuration output
- Canonical workspace confinement, process isolation, permission checks, cancellation, and timeouts
