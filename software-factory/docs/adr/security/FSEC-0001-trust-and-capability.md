# FSEC-0001. Trust and Capability

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0001

## Context

Untrusted data and ambient authority can convert ordinary processing into unauthorized actions.

This root governs trust transitions and privileged effects, independently of confidentiality or resource accounting. No identity provider or deployment-specific parent is selected by the distribution.

## Decision

Trust transitions validate data, origin and permission before granting narrowly scoped capabilities.

R1 [5]: Boundary parsers MUST validate untrusted input before constructing trusted domain values or performing privileged effects.
R2 [5]: Privileged operations MUST authenticate their origin and authorize the specific action on the specific resource.
R3 [5]: Components MUST receive only required capabilities; authentication or authorization failures MUST NOT silently downgrade to permissive access.

## Consequences

+ becomes easier: reviewers can locate validation, authentication and action-specific authorization boundaries.
− becomes harder: adopters must supply threat models, identity integration and narrowly scoped capabilities.
risks/migration: structurally valid input is not automatically trustworthy or authorized.

Evidence: [SOURCES.md](../../../SOURCES.md), Repository families, attributes the trust/capability synthesis and excludes source-specific topology. Review trust transitions and denied-operation tests against R1–R3. No adopter authentication, authorization or deployment security was tested here.
