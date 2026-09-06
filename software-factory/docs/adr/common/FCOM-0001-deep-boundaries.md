# FCOM-0001. Deep Boundaries

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0001

## Context

Interfaces that expose implementation detail distribute change costs across otherwise independent callers.

## Decision

Modules hide cohesive complexity behind contracts aligned with their reasons to change.

R1 [5]: Module interfaces SHOULD expose domain operations rather than reproduce internal storage or transport structure.
R2 [5]: Dependency direction MUST keep domain policy independent of replaceable infrastructure through explicit boundary contracts.
R3 [5]: Boundary reviews MUST examine callers and callees for leaked invariants, duplicated responsibilities and unnecessary configuration.

## Consequences

Smaller public surfaces improve replaceability, but boundary design requires examining connected modules together.
