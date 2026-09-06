# AFM-0020. Parent-Edge Tree Model

Date: 2026-05-01
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0009, AFM-0001

## Context

AFM-0009:R2 separates roots from branches and is the constraining parent;
this decision chooses one References target as structural parent while
preserving other citations. Configuration selects crate obligations; ancestry
groups them rather than defining their semantic applicability. Reviewers must
still distinguish genuine constraints from incidental citations.

## Decision

Every non-Root ADR's structural parent is the **first** target listed
in its `References:` field. Other forward links — additional
`References:`, `Refines:`, `Supersedes:` — are secondary citations
that contribute argument and history but do not place the ADR in the
tree.

R1 [5]: Treat the first `References:` target as the structural parent;
  ignore additional References, Refines, Supersedes, and reverse verbs
  for parent-edge construction
R2 [5]: Recognize a Root ADR by its `Root:` self-reference; Roots
  have no structural parent and anchor a domain subtree
R3 [5]: Suppress L011 only when `Parent-cross-domain: PREFIX-NNNN —
  reason` matches the first References target exactly; mismatched
  declarations fire L018 and dangling targets fire L019
R4 [5]: Render `--tree` output as a per-domain parent-edge forest
  with `[also: …]` annotations for secondary citations and a per-domain
  orphan section for unreachable ADRs
R5 [5]: Classify the structural parent by status: L012 when Draft or
  Proposed (advisory waypoint, chain flows through); L017 when
  Superseded; L021 when Rejected or Deprecated, which cannot anchor a
  live chain; L022 when the status is unparseable and L023 when it is
  absent, both reporting unknown liveness rather than a pass

### Reference ordering

Specialized parents come first, foundational citations after. The
invalidation test: if removing the first target leaves the ADR
intact as a standalone decision, the first target is too weak —
promote a more constraining parent.

```
References: CHE-0006, COM-0018      ← CHE-0006 is the parent
Supersedes: CHE-0027                ← does not affect parent edge
```

### Diagnostics

| ID   | Severity | Trigger |
|------|----------|---------|
| L010 | warning  | Non-Root ADR has no `References:` (no parent) |
| L011 | warning  | First `References:` target is in a different domain |
| L012 | warning  | First `References:` target is Draft or Proposed |
| L013 | warning  | Parent-edge graph contains a cycle |
| L014 | warning  | Parent chain does not terminate at a Root |
| L015 | warning  | First reference is a Root while same-domain non-Root candidates exist |
| L016 | warning  | Structural parent's tier is lower-leverage than child's |
| L017 | warning  | First `References:` target is Superseded |
| L018 | warning  | `Parent-cross-domain` ID does not match first References target |
| L019 | warning  | `Parent-cross-domain` target ADR does not exist |
| L021 | warning  | First `References:` target is Rejected or Deprecated |
| L022 | warning  | First `References:` target has an unparseable status |
| L023 | warning  | First `References:` target has no `Status:` line |

L015 and L016 are heuristics — suppression is a judgment call,
typically by reordering references rather than adding configuration.

L022 and L023 report that the parent's liveness could not be
determined. An undetermined parent is not a sound one; fix the
parent's `Status:` line rather than reading silence as approval.

## Consequences

- Easier: a single ordered reference determines structural placement.
- Harder: reference reordering changes parentage and needs semantic review.
- Risks: heuristics cannot prove parent suitability; terminal or unknown
  ancestry must not be mistaken for a live root. Retired ADRs grant no
  current exception to reference-ordering guidance.

Source evidence: `src/nav.rs:71–173` constructs parent projections;
`src/context.rs:172–244,337–364` groups eligible rules with an unclaimed
fallback. These are graph mechanics, not proof of architectural entailment.
AFM-0039:R6 preserves eligible descendants when non-live ancestry is severed.
