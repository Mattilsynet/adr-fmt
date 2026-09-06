# FCOM-0005. Verification and Authority

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0005

## Context

Duplicated specifications drift, and passing checks can conceal that the relevant behavior was never exercised.

## Decision

One authoritative representation governs each invariant, with executable evidence at its consumer boundary.

R1 [5]: Derived representations MUST identify their authority and have a consistency check or generation path.
R2 [5]: Behavioral changes SHOULD demonstrate an intended failing test before the implementation and passing relevant tests afterward.
R3 [5]: New or changed enforcement guards MUST demonstrate violation, failure, restoration and clean execution; unavailable checks MUST remain explicit gaps.

## Consequences

Evidence becomes reproducible, but green syntax checks alone cannot establish semantic correctness or coverage.
