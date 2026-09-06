# FCOM-0004. Mutation Ownership

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0004

## Context

Concurrency correctness depends on authority over mutation, not merely the presence of synchronization primitives.

This root defines mutation authority independently of any actor, storage or runtime topology. Source ownership concepts support the rationale without importing a parent contract or mandatory single-writer design.

## Decision

Each mutable resource has an explicit ownership and coordination contract.

R1 [5]: Stateful components MUST identify mutation authority, ownership transfer and the serialization boundary for conflicting operations.
R2 [5]: Designs SHOULD prefer partitioned ownership and immutable snapshots; shared mutation MUST state its synchronization invariants.
R3 [5]: Ownership transfer MUST prevent stale authority from mutating protected state; local locks MUST NOT imply distributed exclusion.

## Consequences

+ becomes easier: reviewers can locate mutation authority and transfer boundaries.
− becomes harder: shared-state designs need explicit synchronization and stale-authority reasoning.
risks/migration: local locking cannot establish distributed exclusion; preferred partitioning is not a universal lock prohibition.

Evidence: [SOURCES.md](../../../SOURCES.md), Coverage and conflict review, explains the narrowed ownership adaptation. Review mutation and transfer paths against R1–R3, including stale owners; source lineage is not a concurrency or distributed-fencing proof.
