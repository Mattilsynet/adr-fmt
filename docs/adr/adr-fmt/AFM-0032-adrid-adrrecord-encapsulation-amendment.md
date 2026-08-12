# AFM-0032. AdrId/AdrRecord Encapsulation Amendment to AFM-0026

Date: 2026-08-12
Last-reviewed: 2026-08-12
Tier: S
Status: Accepted

## Related

References: AFM-0026, AFM-0028

## Context

AFM-0026:R1 pinned `model::{AdrRecord, DomainDir, AdrId, ...,
parse_adr_id}` as the library surface but did not constrain field
*visibility*. Every field was `pub`: a consumer could construct an
invariant-violating value directly — a lowercase/empty
`AdrId.prefix`, a `number` above `9999`, or an `AdrRecord` with
mutually inconsistent parser-derived fields.

A first pass at this ADR used `pub(crate)` and claimed the invariants
were thereby "true by construction". False: `pub(crate)` lets every
module in this crate, not only the parser, construct or mutate either
type without validation. Review (linus, round 1) rejected this —
documenting an unenforced guarantee is governance laundering, not
encapsulation. This revision describes what is now implemented:
fields private to `model`, and RFC 2119/8174 uppercase keywords per
AFM-0031.

`adr-fmt` has not shipped to crates.io; tightening visibility now is
pre-release, not breaking. Reconstruction from untrusted input needs
`Result`, not `Option` (error-is-not-a-negative-finding, generalized
to construction). AFM-0029 supersession was rejected — only two
types change, not the whole surface. This ADR amends AFM-0026 via
successor per the AFM-0028 precedent (Route A).

## Decision

Amend AFM-0026 by tightening the field visibility of `AdrId` and
`AdrRecord` to fully private (module-private to `model`, not
`pub(crate)`), adding validated construction and read-only accessors
as the sole external and internal API for those two types, and
reaffirming that the new surface inherits AFM-0028's trait-floor rule.

R1 [5]: `AdrId` and `AdrRecord` fields MUST be private to the `model`
  module — not `pub`, not `pub(crate)`. No consumer, internal or
  external, MAY construct or field-access either type directly; every
  read or construction MUST go through a `model` accessor or
  constructor. Corrects this ADR's first draft, which used
  `pub(crate)` and incorrectly claimed true-by-construction.

R2 [5]: The sole external `AdrId` constructors ARE `try_new(prefix,
  number) -> Result<Self, AdrIdError>` and `TryFrom<&str>`. Both MUST
  validate the AFM-0026:R1 invariants and reject violations with
  `AdrIdError`. `parse_adr_id` / `parse_adr_id_from_filename_stem`
  keep their `Option` signatures (already-pinned surface).
  `AdrIdError` MUST be re-exported at the crate root
  (`adr_fmt::AdrIdError`) so external callers can name and handle it —
  widening AFM-0026:R1's pinned set under AFM-0026:R5.

R3 [5]: `AdrRecord` has no external constructor — `parse_domain` /
  `parse_stale` are the only external routes to an instance. The
  parser's own path is one narrow `pub(crate)` constructor
  (`AdrRecord::from_parser_fields`), not a general builder. Every
  `AdrRecord` field MUST have a read accessor in `model`; a new field
  MUST add its accessor in the same change.

R4 [5]: `AdrIdError` — the error type returned by R2's constructors —
  inherits the AFM-0028:R1 trait floor by construction
  (`core::fmt::Display`, `core::fmt::Debug`, `std::error::Error`) as a
  new type added to the AFM-0026:R1 surface under AFM-0026:R5. No
  separate ADR is needed to establish this per AFM-0028:R4.

R5 [5]: The trait surfaces, accessor signatures, and constructor
  signatures introduced by this ADR are part of the v0.1 semver
  contract per the extension of AFM-0026:R3. New accessors or trait
  impls MAY be added in minor versions; none introduced here MAY be
  removed or reshaped without a successor ADR.

## Consequences

+ becomes easier: no consumer, internal or external, can construct an
  `AdrId` or `AdrRecord` that violates AFM-0026:R1's documented
  invariants; the type system enforces what prose previously
  asserted and what a `pub(crate)` first draft only pretended to
  enforce.
+ becomes easier: `AdrIdError` is nameable and exhaustively matchable
  by external callers via the crate-root re-export, closing the gap
  the first draft left open.
− becomes harder: every new `AdrRecord` field needs its accessor in
  the same change (R3); a genuinely new external constructor for
  either type needs a successor ADR.
risks/migration: `adr-srv`'s own local `AdrId` type is unaffected — it
  touches `adr_fmt::AdrId` only via `Display`/`FromStr`
  (`.to_string()`/`.parse()`), never direct fields, confirmed by
  read-only reconnaissance of `crates/adr-srv/` at authoring time.
  `adr-srv`'s direct field reads of `adr_fmt::AdrRecord` (`record.id`,
  `.title`, `.date`, `.last_reviewed`, `.tier`, `.status`,
  `.file_path`, `.relationships` in `crates/adr-srv/src/scrape.rs`) are
  a latent, accepted break until `adr-srv` migrates those seven call
  sites to the R3 accessors — a mechanical `.field` → `.field()`
  change per site, out of scope for the mission producing this ADR.
</content>
