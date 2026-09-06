# FCOM-0002. Explicit State and Errors

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0002

## Context

Sentinels and ambiguous failures force every caller to reconstruct the same hidden state machine.

## Decision

Domain types represent valid states and preserve uncertainty at fallible boundaries.

R1 [5]: APIs SHOULD use discriminated states and validated constructors instead of sentinel strings or independently mutable flags.
R2 [5]: Fallible observations MUST distinguish negative findings from unavailable, unauthorized or indeterminate evidence.
R3 [5]: Error elimination MUST preserve domain meaning; retries, defaults and empty results MUST NOT disguise failed observations as success.

## Consequences

Callers handle fewer impossible combinations without sacrificing information needed for recovery or accurate verdicts.
