# FCOM-0001. Deep Boundaries

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0001

## Context

Interfaces that expose implementation detail distribute change costs across otherwise independent callers.

This root defines module-boundary obligations independently of state representation, mutation ownership or evolution policy. Those concerns can inform a review without becoming a structural parent.

## Decision

Modules hide cohesive complexity behind contracts aligned with their reasons to change.

R1 [5]: Module interfaces SHOULD expose domain operations rather than reproduce internal storage or transport structure.
R2 [5]: Dependency direction MUST keep domain policy independent of replaceable infrastructure through explicit boundary contracts.
R3 [5]: Boundary reviews MUST examine callers and callees for leaked invariants, duplicated responsibilities and unnecessary configuration.

## Consequences

+ becomes easier: callers depend on domain contracts rather than replaceable internals.
− becomes harder: review must inspect connected callers and callees, not just interface signatures.
risks/migration: an abstraction can add indirection without hiding meaningful complexity; R1 remains a recommendation.

Evidence: [SOURCES.md](../../../SOURCES.md), Repository families, maps boundary and information-hiding synthesis. Interface and call-site review against R1–R3 is the evidence surface; no adopter interface or replaceability experiment was inspected here.
