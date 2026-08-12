# Security Policy

## Reporting a Vulnerability

If you discover a security issue in adr-fmt, please report it privately
rather than filing a public issue.

Send a report to the repository owner via GitHub:
<https://github.com/acje>. Include:

- a description of the issue,
- reproduction steps or a minimal proof of concept,
- any known mitigations.

We will acknowledge receipt within a reasonable window, investigate, and
coordinate disclosure once a fix is available.

## Scope

In scope:

- supply-chain concerns flagged by `cargo audit` / `cargo deny` (policy
  in `deny.toml`),
- input handling in the ADR parser/validator when run over untrusted ADR
  corpora.

Out of scope:

- issues in third-party dependencies for which an upstream advisory
  already exists — please file upstream first,
- adr-fmt is read-only at runtime and has `#![forbid(unsafe_code)]`;
  report findings nonetheless if you see something unexpected.
