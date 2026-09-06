# AFM-0037. Guards Are Proven By Planted Failure

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0035, AFM-0027, AFM-0026

## Context

This repository defends its invariants with executable guards.
`tests/guidelines_parity.rs` and `tests/public_surface_coupling.rs`
walk the source with `syn`; `src/guidelines_golden.rs` pins two
rendered governance outputs byte for byte;
`tests/param_order_determinism.rs` falsifies ordering across twenty
spawned processes.

Each was built from scratch, and each arrived independently at the
same habits: prove the guard by planting a violation that compiles,
claim only the invariant the mechanism reaches, and fail loudly when
the guard's own inputs go empty. The habits are practised and
unwritten, so every new guard re-derives them, and the proposal to
delete the goldens as duplicating the rule registry keeps returning.

## Decision

Ratify the practice the existing guards already follow.

R1 [5]: A guard MUST NOT land or change without a recorded four-step
  proof — plant a violation, observe the failure, revert, observe
  clean — with all four exit codes recorded in the pull request or
  its bead

R2 [5]: The planted violation MUST compile. A plant the compiler
  rejects proves only that the compiler works, so a guard whose proof
  rests on one is unproven

R3 [5]: A guard MUST claim only the invariant its mechanism actually
  reaches. Where that is narrower than the invariant wanted, the
  guard and any ADR citing it MUST state the narrow one

R4 [5]: A guard MUST fail rather than pass when its own inputs are
  empty, its exemption is unbound, or its expected artefact is
  unreadable. Silent emptiness is a false clean, not a pass

R5 [5]: A rendered-output golden MUST NOT be removed as redundant
  with the registry it renders. Measured: a catalog identifier typo
  leaves the parity guard green and only the byte pin fails

R6 [5]: Regenerating a golden MUST NOT be a passing path. The
  regenerating run rewrites and then fails, so no single command can
  both change the expected output and report success

R7 [5]: An `AFM-NNNN` citation in a literal Rust doc attribute under
  `src/` or `tests/` MUST name a live ADR, and each rule id in a
  recognized suffix form MUST exist as a rule line. Existence only,
  never entailment; string literals, identifiers and generated docs
  stay out of reach, and forms the guard detects but cannot read fail
  closed

R8 [5]: An `AFM-NNNN` citation in an assertion, panic or expect
  message argument under `src/` or `tests/` MUST name a live ADR, and
  each rule id it carries MUST exist as a rule line. Argument position
  discriminates, so conditions and operands are data; attribute
  streams and unparsable macro bodies are residual and MUST be
  declared, under R4's floors

## Consequences

A guard's cost is stated up front: it is finished not when it is
green, but when its failure has been seen and written down. That is
one extra plant-fail-revert cycle per guard, and it is the cycle that
separates a guard from a decoration. R1 extends AFM-0035:R4 from
meaning-preserving changes to the guards themselves.

R3 keeps the corpus honest about `syn`, which gives syntax and not
name resolution.

One gap stays open and unguarded: nothing checks that a cited rule
still says what the citing comment claims. That is entailment, and
`syn` gives syntax.
