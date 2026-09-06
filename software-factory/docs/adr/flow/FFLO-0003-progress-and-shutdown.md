# FFLO-0003. Progress and Shutdown

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0003

## Context

Long-lived services need continued progress without allowing individual work units to monopolize execution. This independent root governs supervision, cancellation and termination regardless of queue or telemetry design. Open-ended service lifetime is distinct from bounded work between checks, so no finite-lifetime parent is implied.

## Decision

Service lifetimes are open-ended, but work between supervision points is bounded.

R1 [5]: Workers MUST bound batches, retries and recursion between shutdown checks and provide deliberate scheduling opportunities; immediately ready awaits MUST NOT count as yielding evidence.
R2 [5]: Shutdown MUST stop admission and supervise draining or cancellation to termination; dropping handles MUST NOT count as stopping tasks.
R3 [5]: Cancellation MUST preserve partial-I/O state and ownership obligations; blocking work MUST have bounded admission and an explicit termination policy.

## Consequences

+ becomes easier: distinguishing supervised termination from merely dropping a handle.

− becomes harder: partial I/O, stalled consumers and blocking workers need explicit shutdown tests and ownership accounting.

risks/migration: cancellation is not external rollback; immediately ready awaits do not demonstrate fairness.

Evidence: [SOURCES](../../../SOURCES.md#primary-external-sources) identifies graceful shutdown as an adaptation, not NASA's scheduler rule. R1–R3 need adopter progress, cancellation and termination evidence; no service was exercised by this documentation review.
