# FSTO-0002. Fencing and Recovery

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTO-0002

## Context

Retries and failover can duplicate effects or revive stale writers unless authority is checked where writes become authoritative. This independent root governs mutation authority during retries and recovery, regardless of the selected durability model. Related storage concerns do not require a backend-specific parent.

## Decision

Recovery reestablishes authority before resuming externally visible mutation.

R1 [5]: Authoritative writes MUST reject stale ownership at the mutation boundary; deployment singleton assumptions MUST NOT substitute for fencing.
R2 [5]: Retry contracts MUST distinguish idempotency, deduplication windows and concurrency fences, including behavior after deduplication expires.
R3 [5]: Recovery tests MUST cover replay, corruption, torn writes and ownership conflicts; migration MUST define cutover and rollback or irreversible recovery boundaries.

## Consequences

+ becomes easier: reviewing stale-writer rejection and recovery boundaries separately from deployment assumptions.

− becomes harder: replay, corruption, torn-write and ownership-conflict tests need representative failure fixtures.

risks/migration: deduplication expiration and irreversible cutovers can invalidate retry or rollback expectations; fencing is not established by a deployment singleton.

Evidence: [SOURCES](../../../SOURCES.md#repository-families) maps recovery concepts without importing JetStream headers or file containers. R1–R3 require adopter mutation-boundary and recovery tests; no failover or corruption experiment is claimed here.
