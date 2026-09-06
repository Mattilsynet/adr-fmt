# FCOM-0003. Reversible Evolution

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0003

## Context

Speculative abstractions and coupled changes increase recovery cost before their value is demonstrated.

This root governs change planning and compatibility, independently of a chosen module or ownership design. No particular design decision is its constraining parent.

## Decision

Evolution proceeds through bounded increments with explicit trade-offs and compatibility obligations.

R1 [5]: Material design decisions MUST compare viable alternatives against current constraints and identify a rollback or migration path.
R2 [5]: Structural tidying and behavioral changes SHOULD remain independently reviewable and reversible.
R3 [5]: Public contract changes MUST identify affected consumers and compatibility evidence; simpler existing mechanisms SHOULD precede speculative extensibility.

## Consequences

+ becomes easier: smaller commitments can be reviewed and reversed separately.
− becomes harder: compatibility assessment and irreversible migrations require consumer coordination and approval.
risks/migration: documenting rollback does not demonstrate that lost data or external effects can be restored.

Evidence: [SOURCES.md](../../../SOURCES.md), Repository families, maps tradeoff and evolution synthesis. Compare design alternatives, consumer inventory and rollback or migration evidence against R1–R3; no adopter migration was executed here.
