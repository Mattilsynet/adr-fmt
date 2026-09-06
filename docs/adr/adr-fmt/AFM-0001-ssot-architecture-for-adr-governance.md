# AFM-0001. Single Source of Truth Architecture for ADR Governance

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

Root: AFM-0001

## Context

ADR governance needs one source for mechanically checkable structure and
configuration, while rationale and semantic review remain judgment. This is
the root decision because it establishes that division of authority rather
than specializing another local rule. Generated guidance can expose checks;
it cannot establish that prose is sufficient, evidence entails an obligation,
or a parent is architecturally appropriate.

## Decision

Adopt a layered SSOT architecture where the `adr-fmt` binary is
the authoritative specification for all invariant ADR rules.

R1 [5]: Bind all invariant rules to the `adr-fmt` binary: template
  structure, naming, relationships, lifecycle states, link integrity,
  and section ordering — at least one check per invariant class
R2 [5]: Set `adr-fmt.toml` as the owner of configurable aspects:
  domain definitions, crate mappings, stale directory path, and rule
  parameter overrides
R3 [5]: Emit default-mode guidelines as the generated reference
  document combining code invariants from `adr-fmt` and
  `adr-fmt.toml` configuration into a single authoritative output
R4 [5]: Express every enforceable rule as a validation check in
  `adr-fmt`; constraints that resist validation belong in the
  judgment layer
R5 [5]: Record rationale and judgment guidance in ADR Context and
  Consequences sections, filing them there rather than in standalone
  governance documents — exactly one location per judgment item
R6 [5]: Classify a rule as invariant when violating it produces an
  inconsistent corpus regardless of project context and map it as
  configurable otherwise — apply this classification to every new rule
R7 [5]: Locate the `adr-fmt.toml` marker by walking from the current directory
  (canonicalized where possible) toward the filesystem root, testing
  each ancestor for a regular `adr-fmt.toml` file and binding the
  nearest fit as the marker directory; report the corpus absent when
  the root is reached with no fit
R8 [5]: Judge a candidate marker fit only when it parses, its
  `[corpus] root` resolves inside that directory to an existing
  directory, and at least one configured domain directory exists;
  note an unfit candidate on stderr and keep walking, but halt the
  walk at one that is unparseable, declares a duplicate rule id, or
  leaves domain existence indeterminate

## Consequences

- Easier: `adr-fmt` without flags emits configured governance or setup
  guidance; authors need no independently maintained template.
- Harder: invariant changes require code, catalog and guidance maintenance;
  semantic sufficiency still requires review under AFM-0001:R4.
- Risks: nearest-fit discovery may intentionally skip unsuitable configs;
  present unusable markers must not masquerade as absence (AFM-0039:R1).

Source evidence: `src/lib.rs:397–427,484–574` selects guidance and markers;
`src/guidelines.rs:414–477` renders structure and catalog entries.
`tests/guidelines_parity.rs` checks mechanical parity, not judgment quality
or foreign-policy entailment. No authoring-effectiveness measurement is claimed.
