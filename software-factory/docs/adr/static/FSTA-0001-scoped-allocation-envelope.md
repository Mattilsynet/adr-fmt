# FSTA-0001. Scoped Allocation Envelope

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTA-0001

## Context

No-allocation claims are meaningful only for a specified phase, workload and counted allocation surface. This independent root defines the opt-in measurement envelope without requiring a particular pool or allocator. Domain selection supplies applicability, not a whole-program allocation ban or parent implementation.

## Decision

The optional static profile establishes a scoped no-allocation contract, not a whole-program default.

R1 [5]: Adopters MUST define the measured phase, workload envelope, allocation events counted and permitted initialization before claiming no allocation.
R2 [5]: No-allocation phase capacity MUST be reserved before the measured phase; excess work MUST follow a declared rejection or degradation protocol without hidden growth.
R3 [5]: No-allocation instrumentation MUST cover normal, saturated, error, cancellation and shutdown paths, naming build settings and excluded runtime or dependency activity.

## Consequences

+ becomes easier: assessing allocation claims against a named phase and workload.

− becomes harder: capacity reservation and instrumentation must cover saturation and failure paths.

risks/migration: startup heap reservation differs from static storage; persistent output growth requires a separate budget and claim boundary.

Evidence: [SOURCES](../../../SOURCES.md#primary-external-sources) distinguishes startup reservation, fixed capacity and growing output. R1–R3 require scoped adopter instrumentation; no zero-allocation run, allocator recommendation or whole-process memory bound is supplied.
