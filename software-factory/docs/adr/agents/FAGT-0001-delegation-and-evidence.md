# FAGT-0001. Delegation and Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FAGT-0001

## Context

Automated delegation amplifies authority and context mistakes unless task boundaries survive handoffs.

## Decision

The optional agent profile separates decision authority from bounded execution and durable evidence.

R1 [5]: Delegated missions MUST carry objective, success criteria, scope, abort conditions, budget and rollback or recovery instructions.
R2 [5]: Executors MUST respect source and mutation permissions; blocked capabilities MUST be reported rather than bypassed through another tool.
R3 [6]: Cross-agent evidence MUST have a durable, readable body with provenance; pointers MUST be checked before handoff and MUST NOT expose secrets.

## Consequences

Repositories can choose their own agents and evidence store without losing accountability across execution boundaries.
