# FCOM-0004. Mutation Ownership

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0004

## Context

Concurrency correctness depends on authority over mutation, not merely the presence of synchronization primitives.

## Decision

Each mutable resource has an explicit ownership and coordination contract.

R1 [5]: Stateful components MUST identify mutation authority, ownership transfer and the serialization boundary for conflicting operations.
R2 [5]: Designs SHOULD prefer partitioned ownership and immutable snapshots; shared mutation MUST state its synchronization invariants.
R3 [5]: Ownership transfer MUST prevent stale authority from mutating protected state; local locks MUST NOT imply distributed exclusion.

## Consequences

Ownership becomes reviewable without mandating actors, a particular runtime, or a universal prohibition on locks.
