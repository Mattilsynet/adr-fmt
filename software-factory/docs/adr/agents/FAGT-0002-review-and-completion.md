# FAGT-0002. Review and Completion

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FAGT-0002

## Context

Self-reported completion can conceal skipped integration checks, unresolved review findings and abandoned coordination state.

## Decision

Completion requires scoped verification, independent review where warranted and explicit lifecycle closure.

R1 [5]: Verification MUST distinguish increment, dependent-component and delivery-boundary coverage, preserving producer exit status and naming unrun checks.
R2 [5]: Reviews MUST scale evidence to risk without weakening correctness; security, unsafe and enforcement changes MUST receive independent adversarial review.
R3 [5]: Completion reports MUST include tested artifact identity, commands, outcomes and unresolved findings; the decision owner MUST accept handoff before mission state is closed.

## Consequences

Review and closure remain explicit costs; no particular tracker, role name or commit automation is required.
