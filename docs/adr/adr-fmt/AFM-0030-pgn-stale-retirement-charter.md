# AFM-0030. PGN Stale-Retirement Charter

Date: 2026-06-12
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0022, AFM-0009, AFM-0029, AFM-0008

## Context

This charter governs only the pardosa-domain GEN/PAR reconciliation into PGN,
not general AFM retirement. AFM-0022:R1 is its parent because retirement must
use stub form. AFM-0029:R1 constrains atomicity; R5 here records the scoped
exception to AFM-0029:R3's reciprocal successor declaration, not a general waiver.
AFM-0009:R1 constrains verbs and AFM-0008:R3 preserves identities.

The PGN corpus, consolidation inventory and phase evidence are unavailable in
this checkout. Completion and former classification counts are not independently
established here. The charter remains Accepted; absence of external evidence
neither retires it nor establishes that its application condition is satisfied.

## Decision

R1 [5]: For the pardosa-domain GEN/PAR reconciliation, the `adr-fmt.toml`
  stale-retirement deferral is lifted subject to the PGN consolidation
  completion condition in AFM-0030:R2.

R2 [5]: Pardosa-domain GEN/PAR retirement requires completed PGN consolidation:
  PGN-0001 through PGN-0014 are Accepted and carry the non-conflicting
  rescue and legacy Solon material.

R3 [5]: In the pardosa-domain GEN/PAR retirement, a stale ADR gets
  `Status: Superseded by PGN-NNNN` only when the whole ADR was carried
  into exactly one PGN.

R4 [5]: In the pardosa-domain GEN/PAR retirement, partial supersession,
  two-PGN splits, many-to-one reorganizations and dropped-subject ADRs
  receive narrative-only `## Retirement` prose naming what carried and dropped.

R5 [5]: Lineage for pardosa-domain GEN/PAR retirement lives only on the retired ADR's
  `Status:` field; active PGN ADRs gain no reciprocal `Supersedes:` edge.

R6 [5]: In the pardosa-domain GEN/PAR retirement inventory, rows without a
  clean successor, undecidable rows and non-atomic rows use narrative
  retirement and never fabricate a PGN edge.

## Consequences

+ becomes easier: GEN/PAR retirement has an explicit, narrowly scoped disposition.

− becomes harder: authors must preserve the atomicity evidence; a tempting
  partial or reciprocal edge is now an explicit policy violation.

risks/migration: reviewers must obtain external consolidation evidence before
  applying the charter; local lint cannot establish PGN absorption.

Local evidence: `adr-fmt.toml:23–33` configures AFM only;
`src/parser.rs:734–761` checks relationship targets, not historical absorption.
The foreign inventory and its acceptance evidence remain outside this review.
