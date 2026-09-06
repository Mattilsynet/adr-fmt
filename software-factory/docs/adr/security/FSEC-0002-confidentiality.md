# FSEC-0002. Confidentiality

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0002

## Context

Sensitive information can escape through transport, diagnostics and retention even when primary access controls work.

## Decision

Data classification governs exposure and retention across both normal and failure paths.

R1 [5]: Systems MUST classify sensitive data and define access, retention and deletion obligations for each storage and diagnostic surface.
R2 [5]: Sensitive data crossing untrusted networks MUST use authenticated encryption with peer verification and managed credential rotation.
R3 [5]: Logs and public errors MUST exclude secrets and unnecessary sensitive content; diagnostic detail MUST respect the recipient's authorization.

## Consequences

Operators retain useful diagnostics only through deliberately designed redaction and access-controlled evidence channels.
