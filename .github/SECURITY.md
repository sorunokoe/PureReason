# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x   | ✅ Yes     |
| < 0.2   | ❌ No      |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security issues privately via one of these channels:

1. **GitHub Security Advisories** (preferred) — use the
   [Report a vulnerability](../../security/advisories/new) link in the
   Security tab of this repository.
2. **Email** — send a description to `security@sorunokoe.dev`
   (replace with actual address before publishing).

Include in your report:
- A description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected version(s)
- Any suggested mitigations

## Response Timeline

| Stage | Target |
|---|---|
| Initial acknowledgement | 48 hours |
| Triage and severity assessment | 5 business days |
| Fix or mitigation | 30 days (critical), 90 days (moderate) |
| Public disclosure | After fix ships |

We follow [coordinated disclosure](https://en.wikipedia.org/wiki/Coordinated_vulnerability_disclosure).
Credit is given to reporters in the release notes unless they prefer to remain anonymous.

## Scope

This project is a **pure deterministic verifier** — it contains no LLM, no AI model,
and no network calls in its core library. The primary attack surfaces are:

- **Rust CLI / library**: memory safety (Rust mitigates most classes), panic on
  malformed input, resource exhaustion via large inputs
- **Python package** (`pureason/`): unsafe eval in arithmetic solver — mitigated by
  `_safe_eval` (whitelist of operators/literals only, no builtins)
- **REST API** (`pure-reason-api`): authentication bypass, unbounded input size,
  excessive trust receipt storage

Out of scope: benchmark scripts that process local files, example code, documentation.
