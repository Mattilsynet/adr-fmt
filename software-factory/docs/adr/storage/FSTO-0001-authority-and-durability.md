# FSTO-0001. Authority and Durability

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTO-0001

## Context

Storage guarantees become ambiguous when acknowledged writes, durable state and derived reads share one vocabulary.

## Decision

The optional storage profile requires explicit authority and failure-model contracts.

R1 [5]: Stores MUST identify authoritative state, acknowledgment and durability boundaries, including crash and partial-write behavior.
R2 [5]: Read interfaces MUST declare consistency, permitted staleness and any read-after-write fence rather than imply stronger guarantees from caches.
R3 [5]: Storage selection MUST justify its failure model and recovery objectives; no consistency family, backend or wire format is mandated by this profile.

## Consequences

Adopters choose storage semantics explicitly instead of inheriting upstream event-store or deployment assumptions.
