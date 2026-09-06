# FSEC-0003. Resource Lifecycle

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0003

## Context

Finite local containers do not bound aggregate retention or work across concurrent requests.

## Decision

Resource safety is a scoped accounting contract, not a blanket allocation ban.

R1 [5]: Adopters MUST define workload, lifecycle and aggregate budgets in bytes, items, tasks, attempts, depth and time where applicable, with explicit exhaustion behavior.
R2 [5]: Accounting MUST include waiting producers, backing capacity and completed results; acquisition MUST precede retention and spawning, with checked arithmetic and ownership-bound release through errors and cancellation.
R3 [5]: Evidence MUST cover saturation and concurrency; process-memory claims MUST account for runtime, allocator, stacks and kernel resources or explicitly exclude them.

## Consequences

Unknown capacities remain adopter decisions; application accounting alone does not prove immunity to exhaustion.
