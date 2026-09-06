# FFLO-0001. Admission and Backpressure

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0001

## Context

Queues merely relocate overload unless admission accounts for upstream waiters and downstream capacity.

## Decision

The optional flow profile admits work before committing its resource envelope.

R1 [5]: Ingestion MUST reserve item, byte and task capacity before spawning or retaining work, composing limits across concurrent producers.
R2 [5]: Waiting admission MUST bound waiter count, retained payload and deadline; exhaustion MUST choose an explicit reject, drop, disconnect or degrade result.
R3 [5]: Shaping and batching MUST preserve required ordering; retries MUST consume bounded attempts, time and resource budgets.

## Consequences

Overload becomes a visible protocol outcome rather than hidden memory growth or indefinite waiting.
