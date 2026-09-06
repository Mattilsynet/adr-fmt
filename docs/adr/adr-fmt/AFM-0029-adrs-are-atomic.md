# AFM-0029. ADRs Are Atomic

Date: 2026-05-20
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0022

## Context

An ADR is the unit of architectural commitment. It is either in force
or retired; there is no partial state. Allowing some rules of an ADR
to be superseded while others remain in force splits the unit, makes
authority transfer ambiguous, and forces every reader to cross-check
which clauses still apply. The corpus is easier to reason about when
the ADR is the atom of supersession.

AFM-0022:R1 is the constraining parent: a complete replacement needs a
retired representation, not surviving rules hidden in an archive.

## Decision

R1 [5]: `Supersedes:` names whole ADR identifiers only. Clause-level
  forms are prohibited.

R2 [5]: When only some rules of an ADR become obsolete, the ADR is
  amended in place: obsolete rules are deleted, survivors renumbered
  to remove gaps, `Last-reviewed:` bumped. Prior wording is preserved
  by git history. No supersession edge is created.

R3 [5]: When an ADR is fully replaced, the predecessor moves to
  `docs/adr/stale/` per AFM-0022 and the successor declares whole-ADR
  `Supersedes:`. This is the only supersession path.

R4 [5]: `References:` lines may cite clause-level form. Citation
  carries no replacement semantics; only `Supersedes:` is constrained.

## Consequences

+ becomes easier: reading the citation graph — every `Supersedes:`
  edge is a complete authority transfer.

− becomes harder: partial amendments require renumbering and review of
  citations to the surviving rules.

risks/migration: syntactic acceptance does not establish whole-subject replacement.

Evidence: `src/parser.rs:734–761` strips clause qualifiers only for
References; `supersedes_rejects_clause_level_target` at lines 1337–1349
checks P003 and absence of an edge. Retirement truth remains a review judgment.
