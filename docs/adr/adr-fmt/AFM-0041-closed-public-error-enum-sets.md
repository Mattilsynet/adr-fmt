# AFM-0041. Closed Public Error Enum Sets Amendment to AFM-0026

Date: 2026-11-25
Last-reviewed: 2026-11-25
Tier: S
Status: Accepted

## Related

References: AFM-0026, AFM-0029, AFM-0036

## Context

AFM-0026:R1 defines the public error set and is the first parent: without that
exported surface, this closure has no governed subjects. RST-0006 requires closed
public error enums, so an unhandled state is unrepresentable at compile time rather
than a crash or silent default, and an open variant set is incompatible with that
requirement. AFM-0029 gives the in-place amendment path, and AFM-0036:R3/R4 gives
the breaking version transition, making them the recording terms for this settlement.

## Decision

Amend AFM-0026 by closing every public error enum's variant set in the R1
surface, and record the closure as the breaking transition under AFM-0036:R4.

R1 [5]: The variant sets of the R1-set public error enums are closed per
   RST-0006: adding or removing a variant is a breaking change under
   AFM-0036:R3 and R4.

R2 [5]: Any `pub enum` error type declared in `src/` at item level MUST NOT
   carry `#[non_exhaustive]`, naming the seven: `src/config.rs` LoadError,
   `src/config.rs` ResolveCorpusError, `src/containment.rs` ContainmentError,
   `src/model.rs` RuleIdError, `src/model.rs` AdrIdError, `src/parser.rs`
   ParseError and `src/refs.rs` RefsError, noting RuleIdError and RefsError
   are not re-exported.

R3 [5]: Version 0.5.0 records the closure as the breaking transition under
   AFM-0036:R4, dropping `#[non_exhaustive]` from those seven public error
   enums in `src/`.

## Consequences

+ becomes easier: consumers match each variant exhaustively, so an unhandled
  error state is a compile-time error rather than a runtime default.
− becomes harder: adding or removing a public error variant now requires a
  breaking version under AFM-0036:R4, not a minor-version additive change.
risks/migration: the closure forecloses minor-version variant additions on the
  existing seven; a future addition must ride a major transition and a new ADR.

Evidence: the 0.5.0 release carries the version in `Cargo.toml`;
`tests/closed_error_policy.rs` now asserts the zero-violation prohibition over
the seven, replacing its prior per-enum baseline that cited AFM-0026:R6.
