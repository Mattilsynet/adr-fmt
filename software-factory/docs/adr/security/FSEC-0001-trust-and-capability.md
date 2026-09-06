# FSEC-0001. Trust and Capability

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0001

## Context

Untrusted data and ambient authority can convert ordinary processing into unauthorized actions.

## Decision

Trust transitions validate data, origin and permission before granting narrowly scoped capabilities.

R1 [5]: Boundary parsers MUST validate untrusted input before constructing trusted domain values or performing privileged effects.
R2 [5]: Privileged operations MUST authenticate their origin and authorize the specific action on the specific resource.
R3 [5]: Components MUST receive only required capabilities; authentication or authorization failures MUST NOT silently downgrade to permissive access.

## Consequences

Explicit capabilities reduce accidental authority, while applications still need their own threat models and identity providers.
