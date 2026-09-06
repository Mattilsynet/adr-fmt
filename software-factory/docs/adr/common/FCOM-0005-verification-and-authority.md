# FCOM-0005. Verification and Authority

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FCOM-0005

## Context

Duplicated specifications drift, and passing checks can conceal that the relevant behavior was never exercised.

This root governs authority and verification evidence independently of the policy being checked. Other decisions consume that discipline but do not supply its structural parent.

## Decision

One authoritative representation governs each invariant, with executable evidence at its consumer boundary.

R1 [5]: Derived representations MUST identify their authority and have a consistency check or generation path.
R2 [5]: Behavioral changes SHOULD demonstrate an intended failing test before the implementation and passing relevant tests afterward.
R3 [5]: New or changed enforcement guards MUST demonstrate violation, failure, restoration and clean execution; unavailable checks MUST remain explicit gaps.

## Consequences

+ becomes easier: derived output and guard behavior have inspectable evidence.
− becomes harder: failure plants and restoration require isolated fixtures and execution time.
risks/migration: green syntax checks do not establish semantic correctness or coverage; test-first development remains SHOULD in R2.

Evidence: [verify.py](../../../verify.py) checks corpus/profile identity; [test_verify.py](../../../test_verify.py), PlantTests, plants membership violations and restores isolated copies. Run both from the repository root. Observed exits belong in the dated review record; these checks do not prove adopter behavior.
