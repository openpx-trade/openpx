# Security Policy

OpenPX handles private keys, wallet signing, and real financial transactions. We take security seriously.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email **security@openpx.ai** with:

1. A description of the vulnerability
2. Steps to reproduce
3. Potential impact
4. Any suggested fix (optional)

## Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial assessment:** Within 1 week
- **Fix or mitigation:** Dependent on severity, but we aim for:
  - Critical (private key exposure, fund theft): 24-48 hours
  - High (authentication bypass, data leak): 1 week
  - Medium/Low: Next scheduled release

## Scope

The following are in scope:

- Private key handling in all `px-exchange-*` crates
- Authentication flows (RSA-PSS, EIP-191, HMAC, JWT)
- Order signing and submission
- Credential storage and transmission
- Dependencies with known CVEs

## Best Practices for Users

- Never commit `.env` files or private keys to version control
- Use environment variables or secure vaults for credentials
- Run with minimal permissions (read-only API keys for market data)
- Keep dependencies up to date (`cargo update`)
