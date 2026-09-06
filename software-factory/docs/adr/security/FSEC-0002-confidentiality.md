# FSEC-0002. Confidentiality

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0002

## Context

Sensitive information can escape through transport, diagnostics and retention even when primary access controls work.

This root governs information exposure and retention across surfaces, independently of a particular authentication design. Trust policy informs review but is not an invented parent obligation.

## Decision

Data classification governs exposure and retention across both normal and failure paths.

R1 [5]: Systems MUST classify sensitive data and define access, retention and deletion obligations for each storage and diagnostic surface.
R2 [5]: Sensitive data crossing untrusted networks MUST use authenticated encryption with peer verification and managed credential rotation.
R3 [5]: Logs and public errors MUST exclude secrets and unnecessary sensitive content; diagnostic detail MUST respect the recipient's authorization.

## Consequences

+ becomes easier: classification connects transport, retention and diagnostic exposure decisions.
− becomes harder: redaction and access-controlled evidence channels complicate diagnosis and credential operations.
risks/migration: configuration alone does not prove deletion, correct peer verification or absence of secret leakage.

Evidence: [SOURCES.md](../../../SOURCES.md), Repository families, maps confidentiality synthesis. Review data inventories, retention/deletion paths, transport settings and recipient-specific diagnostics against R1–R3; no sensitive-data scan or cryptographic deployment assessment was executed here.
