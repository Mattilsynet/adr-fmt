# AFM-0009. Minimal Relationship Vocabulary With Three Verbs

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0001

## Context

AFM-0001:R1 owns relationship invariants and is the constraining parent.
Three verbs distinguish root declaration, citation and replacement without
requiring authors to classify every rhetorical relationship. Structural
parentage further specializes References under AFM-0020:R1. Historical
cherry-pit analysis is not available as independently inspected evidence;
this decision makes no completeness claim about every possible relationship.

## Decision

Restrict the relationship vocabulary to exactly three verbs. All
other verbs are legacy and produce a deprecation warning.

R1 [5]: Permit only Root, References, and Supersedes as
  relationship verbs; legacy verbs trigger a warning (L006)
R2 [5]: Declare each ADR as either a tree root or a branch —
  Root and References are mutually exclusive per L009
R3 [5]: Anchor the Supersedes verb to a target ADR that carries
  `Superseded by PREFIX-NNNN` status (enforced by L003)
R4 [5]: Match Root target to the ADR's own ID; validate
  this constraint via L008 on every corpus scan
R5 [5]: Permit multiple References entries to support cross-cutting
  concerns drawing on several prior decisions — at least one
  References entry required per non-root ADR

## Consequences

- Easier: citations use one verb; replacement and root declaration stay distinct.
- Harder: authors express finer relationships in prose rather than new verbs.
- Risks: a non-root has at most one structural parent, not guaranteed membership
  in a live tree; missing, cyclic or terminal ancestry needs diagnosis.

Source evidence: `src/rules/links.rs:146–212` checks replacement, self-roots
and legacy verbs; `src/nav.rs:71–106` selects the first References target.
AFM-0020:R5 governs liveness. Mechanical graph construction cannot establish
that the selected parent actually constrains the child.
