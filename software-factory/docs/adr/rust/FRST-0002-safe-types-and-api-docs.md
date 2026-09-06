# FRST-0002. Safe Types and API Documentation

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FRST-0002

## Context

Rust ownership can encode contracts that prose and repeated runtime checks leave vulnerable to drift. This independent root governs Rust construction and documentation boundaries regardless of compiler pin or dependency selection; it does not require an invented build-policy parent.

## Decision

Safe types carry invariants; documentation serves actual API and safety contracts.

R1 [5]: Rust implementations SHOULD use newtypes, enums and RAII to represent legal states and resource lifetimes; unsafe code MUST require a separately reviewed safety contract.
R2 [5]: Rust source MUST NOT contain non-doc comments; doc comments MUST serve public API, unsafe safety or executable documentation contracts, not private rationale.
R3 [5]: Public Rust documentation MUST describe applicable errors, panics and safety preconditions; durable design rationale MUST live in decisions or review evidence.

## Consequences

+ becomes easier: understanding legal states and public contracts through types and names.

− becomes harder: construction routes, unsafe contracts and documentation necessity require individual review.

risks/migration: a safe constructor does not validate alternate construction or mutation routes; a comment linter cannot prove documentation is necessary.

Evidence: [SOURCES](../../../SOURCES.md#coverage-and-conflict-review) records the private-rationale exclusion. R1–R3 need adopter construction-route, safety and rustdoc review; no compile-fail or unsafe-soundness proof is supplied here.
