# AFM-0031. RFC-2119/8174 Requirement-Level Keywords as Normative Vocabulary

Date: 2026-07-16
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0012, GND-0002, GND-0009

## Context

Requirement strength and leverage answer different questions. AFM-0012:R1
is the constraining parent because it supplies the tagged obligations whose
strength this vocabulary expresses. RFC 2119/8174 uppercase keywords distinguish
requirements from recommendations without changing the Meadows tags. Reviewers
still judge intent and exceptions; keyword presence is not semantic proof.
Foreign GND/COM citations retain their supporting role, without a fresh external
entailment check in this local review.

## Decision

The corpus adopts the RFC 2119 / RFC 8174 keyword set as its normative
requirement-level vocabulary, uppercase-only, orthogonal to Meadows layer.

R1 [6]: Use MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD
  NOT, RECOMMENDED, MAY, and OPTIONAL as the corpus's normative
  requirement-level vocabulary wherever a tagged rule states a
  requirement or prohibition
R2 [6]: Treat only the uppercase forms of these keywords as carrying
  normative force per RFC 8174; treat lowercase occurrences of the same
  words as ordinary prose with no normative meaning
R3 [6]: Hold the keyword axis orthogonal to the Meadows-layer axis —
  layer classifies intervention type, keyword classifies requirement
  strength; a high-layer rule does not imply MUST, and a MUST rule
  implies no particular layer
R4 [5]: Reserve keywords for tagged rules expressing a genuine
  requirement or prohibition per RFC 2119 §6 restraint, tying keyword use
  to the COM-0001 complexity budget rather than decorative emphasis
R5 [6]: Apply the keyword vocabulary to new and amended ADRs going
  forward; leave existing ADRs grandfathered and reword them
  opportunistically in place per AFM-0029:R2, with no flag-day migration

Keyword interpretation is a review obligation, not a new parser grammar or
keyword lint. Generated authority lives in `src/guidelines.rs`, not a
standalone template file.

## Consequences

+ becomes easier: reviewers distinguish declared strength from leverage.
− becomes harder: rule authors must learn and consistently apply the
  RFC 2119/8174 vocabulary instead of free-form phrasing.
− becomes harder: mixed old/new phrasing persists during the
  grandfather period, so keyword presence cannot yet be treated as
  universal across the corpus.
risks/migration: no flag-day migration — existing ADRs are grandfathered
  and reworded opportunistically in place (AFM-0029:R2) when otherwise
  touched. No new keyword diagnostic is committed here.

Evidence: `src/guidelines.rs:133–178` teaches strength and human judgment;
`src/parser.rs:977–1025` recognizes rule shape, not requirement meaning.
