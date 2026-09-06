# FSEC-0003. Resource Lifecycle

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0003

## Context

Finite local containers do not bound aggregate retention or work across concurrent requests.

This root defines baseline resource accounting, independently of optional flow or static-allocation profiles. Those profiles do not supply a parent or universal capacity; the distribution deliberately leaves workload limits to adopters.

## Decision

Resource safety is a scoped accounting contract, not a blanket allocation ban.

R1 [5]: Adopters MUST define workload, lifecycle and aggregate budgets in bytes, items, tasks, attempts, depth and time where applicable, with explicit exhaustion behavior.
R2 [5]: Resource accounting MUST include waiting producers, backing capacity and completed results; acquisition MUST precede retention and spawning, with checked arithmetic and ownership-bound release through errors and cancellation.
R3 [5]: Resource-budget evidence MUST cover saturation and concurrency; process-memory claims MUST account for runtime, allocator, stacks and kernel resources or explicitly exclude them.

## Consequences

+ becomes easier: reviewers can trace admitted resources through ownership and release.
− becomes harder: aggregate accounting includes waiters, backing capacity and failure paths, not merely queue length.
risks/migration: unknown capacities remain adopter decisions; application accounting does not prove immunity to exhaustion.

Evidence: [SOURCES.md](../../../SOURCES.md), Coverage and conflict review, specifies the accounting/flow/static split and applicable stress cases. Review resource contracts and measured saturation/concurrency evidence against R1–R3; this prose migration measures no high-water mark and supplies no process-memory or latency bound.
