# FGND-0002. Feedback and Deviation

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

Root: FGND-0002

## Context

Plans lose contact with reality when execution reports describe activity rather than observable effects.

This root governs feedback and reassessment, not the contents of a work brief. It remains independently applicable and retireable without making the intent decision its structural parent.

## Decision

Feedback connects delegated action to intent through observable outcomes and explicit escalation boundaries.

R1 [3]: Success criteria MUST name observable outcomes and the evidence needed to distinguish success, failure and incomplete knowledge.
R2 [3]: Executors MAY adapt reversible actions within delegated boundaries but MUST report material deviations to the decision owner.
R3 [3]: Evidence contradicting a load-bearing assumption MUST trigger reassessment before further dependent work proceeds.

## Consequences

+ becomes easier: reports distinguish observed outcomes from activity and incomplete knowledge.
− becomes harder: evidence collection and escalation consume delivery time.
risks/migration: selected observations can miss a material deviation; reporting does not guarantee timely reassessment.

Evidence: [SOURCES.md](../../../SOURCES.md), Repository families, records the ground-policy lineage. Review outcome evidence, deviation reports and reassessment records against R1–R3; no executor-effectiveness measurement is supplied here.
