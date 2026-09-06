# FSTO-0001. Authority and Durability

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTO-0001

## Context

Storage guarantees become ambiguous when acknowledged writes, durable state and derived reads share one vocabulary. This independent root defines storage-contract vocabulary and selection obligations without depending on a recovery implementation. Neither source lineage nor the storage profile imposes a backend parent.

## Decision

The optional storage profile requires explicit authority and failure-model contracts.

R1 [5]: Stores MUST identify authoritative state, acknowledgment and durability boundaries, including crash and partial-write behavior.
R2 [5]: Read interfaces MUST declare consistency, permitted staleness and any read-after-write fence rather than imply stronger guarantees from caches.
R3 [5]: Storage selection MUST justify its failure model and recovery objectives; no consistency family, backend or wire format is mandated by this profile.

## Consequences

+ becomes easier: comparing acknowledgment, durability and read-consistency promises.

− becomes harder: selecting storage requires explicit failure models and recovery objectives.

risks/migration: a cache or successful acknowledgment can be mistaken for a stronger durability guarantee; no event-store topology or backend is inherited.

Evidence: [SOURCES](../../../SOURCES.md#repository-families) records storage lineage and excluded formats/topologies. R1–R3 require adopter authority, crash and read-consistency contracts; source mapping and lint do not test durable writes or recovery.
