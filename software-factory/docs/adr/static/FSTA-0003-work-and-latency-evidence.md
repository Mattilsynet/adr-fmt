# FSTA-0003. Work and Latency Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSTA-0003

## Context

Algorithmic progress, operation counts and elapsed-time deadlines describe different properties of a running system.

## Decision

Predictability claims identify their operation and distinguish abstract work from observed latency.

R1 [5]: Work bounds MUST name the operation, configured capacity and input assumptions; full-capacity scans MUST include their idle-work cost.
R2 [5]: Latency claims MUST identify scheduling, contention, instruction-cost and hardware assumptions; constant work or lock-free progress MUST NOT substitute for deadline evidence.
R3 [5]: Collection complexity MUST be checked per operation; no_std, zero-cost abstractions and inline storage MUST NOT imply whole-program no-allocation guarantees.

## Consequences

Finite benchmarks support only their measured conditions; fixed capacity does not make every operation constant time.
