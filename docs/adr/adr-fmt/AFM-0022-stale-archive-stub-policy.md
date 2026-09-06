# AFM-0022. Stale Archive Stub Policy

Date: 2026-05-01
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0003, AFM-0008, AFM-0009

## Context

Retired bodies can look authoritative while describing removed behavior.
Stubs keep identity, disposition and lineage visible without reissuing obsolete
rules; full rationale remains in git history. AFM-0003:R1 is the first-parent
constraint because stub validation remains advisory. AFM-0008:R3 preserves
identity, and AFM-0009:R3 constrains replacement lineage.

## Decision

Stale ADRs reduce to a stub: the preamble fields, an optional
`## Related` section restricted to `Supersedes:` lineage edges,
and a `## Retirement` section. All other body content is deleted
in the same commit that moves the ADR to stale. Advisory
lint rule (S007) enforces the stub structure positively, and
T007/T008/T009/T010/T016 skip stale ADRs so compliant stubs
remain lint-clean.

R1 [11]: Confine files under `docs/adr/stale/` with a terminal
  `Status:` (`Superseded by X`, `Deprecated`, `Rejected`) to
  the preamble, an optional `## Related` section limited to
  `Supersedes:` edges (the reverse direction lives in the
  `Status:` field), and a `## Retirement` section
R2 [11]: Delete `## Context`, `## Decision`, and `## Consequences`
  sections and any `References:` lines in the same commit that
  moves an ADR to stale; preserve full prior content via git history
R3 [11]: Apply lint rule S007 (severity warning) on any stale ADR
  whose section list or relationship list violates R1, with one
  diagnostic per violation, skipping T007/T008/T009/T010/T016 for
  stale ADRs so the stub form is positively defined by S007 alone

The `## Retirement` body itself is unstructured prose. A
conventional `Superseded-by:` / `Moved-to-stale:` / `Reason:`
triple appears in most stubs as a quick reference, but narrative-
only retirements (see AFM-0010) are also accepted; no rule
parses or validates the retirement-block contents.

## Consequences

- Easier: readers see disposition rather than obsolete rules posing as authority.
- Harder: historical decisions require git access; retirement needs a deliberate
  body reduction and lineage check.
- Risks: a structurally valid stub may still contain misleading retirement prose;
  external warning gates may block non-compliant archives.

Source evidence: `src/rules/template.rs:475–620` checks lifecycle and S007
structure, not the truth of retirement narrative. `src/nav.rs:149–160`
excludes stale-origin parent edges. This source review does not recover or
revalidate every historical decision.
