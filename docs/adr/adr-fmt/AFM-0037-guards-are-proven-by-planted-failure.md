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

AFM-0035:R4 is the constraining parent: executable meaning evidence is useful
only if the check can detect the targeted violation. Guard proof adds that
failure observation without claiming semantic name resolution from syntax.

## Decision

Require failure evidence and explicit scope for executable guards.

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
  with the registry it renders; rule-identifier parity does not establish
  byte-for-byte output stability

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
  message argument under `src/` or `tests/` MUST name a live ADR; each
  rule id in a recognized suffix form MUST exist as a rule line.
  Argument position discriminates: conditions and operands are data;
  attribute streams stay out of reach; recognized message macro bodies
  fail closed on parse failure, under R4's floors

## Consequences

+ becomes easier: reviewers can inspect evidence that a guard detects its target.
− becomes harder: guard changes require a recorded plant/fail/revert/clean cycle.
risks/migration: syntax checks establish citation existence, not entailment or
  complete construction control.

Evidence: `src/guidelines_golden.rs:87–119` rejects regeneration-as-success and
compares bytes; `tests/guidelines_parity.rs:20–52` discloses bypasses and its
trusted base. Citation suites are `tests/adr_citation_existence.rs` and
`tests/adr_message_citations.rs`. This documentation pass changes no guard logic
and claims no fresh planted-failure proof.
