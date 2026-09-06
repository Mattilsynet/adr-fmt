# AFM-0011. Meadows-Aligned Tier Classification

Date: 2026-04-28
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

Supersedes: AFM-0010
References: AFM-0001, GND-0008

## Context

AFM-0001:R1 makes classification part of invariant governance and is the
constraining parent. System-characteristic questions distinguish intervention
type from blast radius: a critical parameter is still a parameter. Tier
classification is separate from AFM-0020:R1 structural parentage. GND-0008
remains a foreign citation whose original evidence is not locally verified;
no effect on model compliance or attention is asserted.

## Decision

Classify ADRs by system characteristic using five tiers aligned
with Meadows' leverage-point hierarchy. Use system-characteristic
framing ("Does this decision define X?") instead of blast-radius
framing ("If this changed, would X change?").

R1 [5]: Classify ADRs using the first-yes-wins method: start at
  S-tier and assign the first tier whose classification question
  yields "yes"
R2 [5]: Frame tier classification questions as "Does this decision
  define X?" to classify by leverage type rather than blast radius
R3 [5]: Map tiers to Meadows' leverage hierarchy: S=Intent (levels
  1-3), A=Self-organization (level 4), B=Design (levels 5-6),
  C=Feedbacks (levels 7-8), D=Parameters (levels 9-12)

## Consequences

- Easier: first-yes classification separates parameters from information flows.
- Harder: authors must judge system characteristics, including non-Rust
  extensibility mechanisms, rather than substitute impact for leverage.
- Risks: a tier label does not prove an appropriate classification or improve
  retrieval consumers' behavior by itself.

Source evidence: `src/rules/template.rs:876–899` implements the asymmetric
T019 comparison required by AFM-0012:R4. Equal or lower-leverage rules pass;
that is mechanical tension assessment, not semantic approval. Reassessment
of existing A/B/C classifications remains an author review responsibility;
no foreign-corpus reassessment is claimed here.
