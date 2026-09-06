# FCOM-0003. Reversible Evolution

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0003

## Context

Speculative abstractions and coupled changes increase recovery cost before their value is demonstrated.

## Decision

Evolution proceeds through bounded increments with explicit trade-offs and compatibility obligations.

R1 [5]: Material design decisions MUST compare viable alternatives against current constraints and identify a rollback or migration path.
R2 [5]: Structural tidying and behavioral changes SHOULD remain independently reviewable and reversible.
R3 [5]: Public contract changes MUST identify affected consumers and compatibility evidence; simpler existing mechanisms SHOULD precede speculative extensibility.

## Consequences

Incremental delivery reduces uncertain commitments, while irreversible migrations still require explicit coordination and approval.
