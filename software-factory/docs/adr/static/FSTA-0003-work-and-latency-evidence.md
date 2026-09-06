# FSTA-0003. Work and Latency Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTA-0003

## Context

Algorithmic progress, operation counts and elapsed-time deadlines describe different properties of a running system. This independent root governs the meaning of predictability evidence rather than allocator selection. Work and latency claims need their own assumptions even when another decision specifies fixed capacity.

## Decision

Predictability claims identify their operation and distinguish abstract work from observed latency.

R1 [5]: Work bounds MUST name the operation, configured capacity and input assumptions; full-capacity scans MUST include their idle-work cost.
R2 [5]: Latency claims MUST identify scheduling, contention, instruction-cost and hardware assumptions; constant work or lock-free progress MUST NOT substitute for deadline evidence.
R3 [5]: Collection complexity MUST be checked per operation; no_std, zero-cost abstractions and inline storage MUST NOT imply whole-program no-allocation guarantees.

## Consequences

+ becomes easier: distinguishing abstract operation bounds from elapsed-time evidence.

− becomes harder: complexity must be checked per operation and latency assumptions recorded explicitly.

risks/migration: finite benchmarks support only measured conditions; fixed capacity does not make every operation constant time, and idle scans still consume work.

Evidence: [SOURCES](../../../SOURCES.md#primary-external-sources) records heapless operation differences and lock-free progress limits. R1–R3 require adopter complexity and latency evidence; no deadline benchmark or universal timing guarantee is supplied.
