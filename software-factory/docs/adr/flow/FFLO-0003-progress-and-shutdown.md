# FFLO-0003. Progress and Shutdown

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0003

## Context

Long-lived services need continued progress without allowing individual work units to monopolize execution.

## Decision

Service lifetimes are open-ended, but work between supervision points is bounded.

R1 [5]: Workers MUST bound batches, retries and recursion between shutdown checks and provide deliberate scheduling opportunities; immediately ready awaits MUST NOT count as yielding evidence.
R2 [5]: Shutdown MUST stop admission and supervise draining or cancellation to termination; dropping handles MUST NOT count as stopping tasks.
R3 [5]: Cancellation MUST preserve partial-I/O state and ownership obligations; blocking work MUST have bounded admission and an explicit termination policy.

## Consequences

Shutdown tests must include stalled consumers and partial effects; cancellation is not external rollback.
