# FAGT-0001. Delegation and Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FAGT-0001

## Context

Automated delegation amplifies authority and context mistakes unless task boundaries survive handoffs. This independent root governs delegation permissions and evidence transfer, not a specific review workflow or tracker. Its authority boundary applies without inventing an agent-role parent hierarchy.

## Decision

The optional agent profile separates decision authority from bounded execution and durable evidence.

R1 [5]: Delegated missions MUST carry objective, success criteria, scope, abort conditions, budget and rollback or recovery instructions.
R2 [5]: Executors MUST respect source and mutation permissions; blocked capabilities MUST be reported rather than bypassed through another tool.
R3 [6]: Cross-agent evidence MUST have a durable, readable body with provenance; pointers MUST be checked before handoff and MUST NOT expose secrets.

## Consequences

+ becomes easier: inspecting task scope and provenance across handoffs.

− becomes harder: durable bodies, pointer checks and permission boundaries add coordination work.

risks/migration: a readable pointer does not establish evidence truth or secret safety; changing stores requires preserving provenance and access boundaries.

Evidence: [SOURCES](../../../SOURCES.md#repository-families) maps delegation concepts while excluding mandatory tools and agent names. R1–R3 need mission, permission and evidence-body review; this corpus review is not a permission-bypass test or secret scan.
