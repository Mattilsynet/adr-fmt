# FSTA-0001. Scoped Allocation Envelope

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTA-0001

## Context

No-allocation claims are meaningful only for a specified phase, workload and counted allocation surface.

## Decision

The optional static profile establishes a scoped no-allocation contract, not a whole-program default.

R1 [5]: Adopters MUST define the measured phase, workload envelope, allocation events counted and permitted initialization before claiming no allocation.
R2 [5]: Capacity MUST be reserved before the measured phase; excess work MUST follow a declared rejection or degradation protocol without hidden growth.
R3 [5]: Instrumentation MUST cover normal, saturated, error, cancellation and shutdown paths, naming build settings and excluded runtime or dependency activity.

## Consequences

Startup heap reservation differs from static storage; persistent output growth requires a separate budget and claim boundary.
