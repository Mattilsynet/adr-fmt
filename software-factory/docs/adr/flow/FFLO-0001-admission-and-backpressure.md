# FFLO-0001. Admission and Backpressure

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0001

## Context

Queues merely relocate overload unless admission accounts for upstream waiters and downstream capacity. This independent root governs the admission boundary rather than telemetry or scheduling implementation. Its applicability does not depend on adopting a particular queue or executor as parent authority.

## Decision

The optional flow profile admits work before committing its resource envelope.

R1 [5]: Ingestion MUST reserve item, byte and task capacity before spawning or retaining work, composing limits across concurrent producers.
R2 [5]: Waiting admission MUST bound waiter count, retained payload and deadline; exhaustion MUST choose an explicit reject, drop, disconnect or degrade result.
R3 [5]: Shaping and batching MUST preserve required ordering; retries MUST consume bounded attempts, time and resource budgets.

## Consequences

+ becomes easier: exposing overload as a protocol outcome before retaining work.

− becomes harder: capacity reservations must compose across producers, waiters and retries.

risks/migration: rejecting or dropping work can violate application semantics unless the adopter chooses the protocol and budgets; no universal limits are supplied.

Evidence: [SOURCES](../../../SOURCES.md#coverage-and-conflict-review) separates admission from aggregate accounting. Review R1–R3 through admission ownership, saturation and retry tests; corpus membership checks do not measure retained memory or prove overload safety.
