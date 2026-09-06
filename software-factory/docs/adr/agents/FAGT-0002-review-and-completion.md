# FAGT-0002. Review and Completion

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FAGT-0002

## Context

Self-reported completion can conceal skipped integration checks, unresolved review findings and abandoned coordination state. This independent root governs acceptance of delivery evidence, whether work was delegated or executed directly. It therefore needs no delegation-specific parent or mandatory tracker.

## Decision

Completion requires scoped verification, independent review where warranted and explicit lifecycle closure.

R1 [5]: Verification MUST distinguish increment, dependent-component and delivery-boundary coverage, preserving producer exit status and naming unrun checks.
R2 [5]: Reviews MUST scale evidence to risk without weakening correctness; security, unsafe and enforcement changes MUST receive independent adversarial review.
R3 [5]: Completion reports MUST include tested artifact identity, commands, outcomes and unresolved findings; the decision owner MUST accept handoff before mission state is closed.

## Consequences

+ becomes easier: distinguishing verified delivery from partial execution or unresolved review.

− becomes harder: risk-scaled independent review and decision-owner acceptance add explicit closure costs.

risks/migration: passing an increment does not establish delivery-boundary coverage; no tracker, role name or commit automation is mandated.

Evidence: [README](../../../README.md#inspect-from-this-checkout) names local checks and [test_verify.py](../../../test_verify.py) exercises membership failures and restorations. These support mechanical corpus verification only; R1–R3 still require scoped execution records, independent review and owner acceptance.
