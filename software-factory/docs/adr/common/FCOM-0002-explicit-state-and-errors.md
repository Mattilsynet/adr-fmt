# FCOM-0002. Explicit State and Errors

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0002

## Context

Sentinels and ambiguous failures force every caller to reconstruct the same hidden state machine.

This root defines state and observation semantics independently of module topology. Boundary design is related, but supplies no required parent for preserving unknown outcomes.

## Decision

Domain types represent valid states and preserve uncertainty at fallible boundaries.

R1 [5]: APIs SHOULD use discriminated states and validated constructors instead of sentinel strings or independently mutable flags.
R2 [5]: Fallible observations MUST distinguish negative findings from unavailable, unauthorized or indeterminate evidence.
R3 [5]: Error elimination MUST preserve domain meaning; retries, defaults and empty results MUST NOT disguise failed observations as success.

## Consequences

+ becomes easier: callers retain information needed for recovery and accurate verdicts.
− becomes harder: explicit unavailable and unauthorized states require consumer handling.
risks/migration: constructor checks alone do not cover alternate construction or mutation routes; R1 does not prohibit genuinely independent flags.

Evidence: [SOURCES.md](../../../SOURCES.md), Coverage and conflict review, explicitly rejects failed-observation masking. Review domain constructors, mutations and failure paths against R1–R3; no universal construction-safety or recovery proof is supplied.
