# FSTO-0002. Fencing and Recovery

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTO-0002

## Context

Retries and failover can duplicate effects or revive stale writers unless authority is checked where writes become authoritative.

## Decision

Recovery reestablishes authority before resuming externally visible mutation.

R1 [5]: Authoritative writes MUST reject stale ownership at the mutation boundary; deployment singleton assumptions MUST NOT substitute for fencing.
R2 [5]: Retry contracts MUST distinguish idempotency, deduplication windows and concurrency fences, including behavior after deduplication expires.
R3 [5]: Recovery tests MUST cover replay, corruption, torn writes and ownership conflicts; migration MUST define cutover and rollback or irreversible recovery boundaries.

## Consequences

Correctness can survive failover without prescribing JetStream headers, file containers or a particular replay implementation.
