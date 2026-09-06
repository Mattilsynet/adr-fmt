# AFM-0004. MADR Template Defined in Code Not as a File

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: A
Status: Accepted

## Related

References: AFM-0001

## Context

AFM-0001:R1 makes the binary authoritative for invariant structure, so it
is the constraining parent of this template representation. Parser types,
validation rules and generated guidance are co-located rather than maintained
as an independent copyable template. That makes structural correspondence
inspectable without claiming that a valid document is a sound decision.

## Decision

Define the MADR template entirely in Rust code. No standalone
template file exists.

R1 [5]: Declare valid ADR structure in the parser module as Rust
  types for required metadata, sections, and vocabularies —
  each validated by rule functions in `adr-fmt`
R2 [5]: Build default-mode guidelines to generate human-readable documentation
  from the same code structures in `adr-fmt` that perform validation
R3 [5]: Commit a structural rule by changing parser, rules, and
  guidelines within the same crate — exactly three co-located code
  changes required; inconsistency becomes a compile-time or test-time
  failure in `adr-fmt`
R4 [5]: Select the MADR format by omitting at least two original
  optional sections (`## Options`, `## Pros and Cons`) where Context
  and Consequences serve the same purpose

## Consequences

- Easier: running `adr-fmt` without flags obtains current generated guidance.
- Harder: structural changes require coordinated parser, rules and guidelines
  updates under AFM-0004:R3, rather than a standalone template edit.
- Risks: mechanical parity cannot judge rationale, parent choice or evidence.

Source evidence: `src/parser.rs:467–542` builds records;
`src/rules/template.rs:435–458,622–690` checks sections;
`src/guidelines.rs:414–477` emits structure/catalog guidance.
`tests/guidelines_parity.rs` is a mechanical correspondence probe, not a
semantic-authoring guarantee.
