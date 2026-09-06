# FRST-0002. Safe Types and API Documentation

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FRST-0002

## Context

Rust ownership can encode contracts that prose and repeated runtime checks leave vulnerable to drift.

## Decision

Safe types carry invariants; documentation serves actual API and safety contracts.

R1 [5]: Implementations SHOULD use newtypes, enums and RAII to represent legal states and resource lifetimes; unsafe code MUST require a separately reviewed safety contract.
R2 [5]: Rust source MUST NOT contain non-doc comments; doc comments MUST serve public API, unsafe safety or executable documentation contracts, not private rationale.
R3 [5]: Public documentation MUST describe applicable errors, panics and safety preconditions; durable design rationale MUST live in decisions or review evidence.

## Consequences

Names and structure carry local meaning; a comment linter cannot prove documentation is semantically necessary.
