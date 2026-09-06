# FSTA-0002. Pool Identity and Reclamation

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTA-0002

## Context

Reusing bounded storage can preserve physical memory safety while invalidating logical identities and lifetime assumptions. This independent root governs reuse authority and identity, independently of the chosen allocation phase or timing claim. A common static profile does not make either neighboring decision its parent.

## Decision

Pool reuse requires explicit identity, ownership and reclamation contracts.

R1 [5]: Recycled handles MUST reject stale identities through generations or equivalent ownership proofs, including exhaustion and generation-overflow behavior.
R2 [5]: Pool capacity accounting MUST include alignment, backing storage and deferred reclamation; dropping a value MUST NOT imply bump capacity is reusable.
R3 [5]: Arena reset MUST require exclusive authority and proof that no live reference survives; exhaustion MUST remain explicit before unsafe reuse.

## Consequences

+ becomes easier: identifying stale-handle and reset obligations before storage reuse.

− becomes harder: generations, overflow and deferred reclamation complicate capacity and lifetime accounting.

risks/migration: fixed capacity does not prevent logical use-after-free, ABA errors or cumulative bump exhaustion.

Evidence: [SOURCES](../../../SOURCES.md#primary-external-sources) distinguishes bump reset and deferred reclamation. R1–R3 need adopter handle-construction/mutation, overflow and exclusive-reset evidence; no allocator safety or live-reference exclusion proof is established by this corpus.
