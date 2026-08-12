# AFM-0032. AdrId/AdrRecord Encapsulation Amendment to AFM-0026

Date: 2026-08-12
Last-reviewed: 2026-08-12
Tier: S
Status: Accepted

## Related

References: AFM-0026, AFM-0028

## Context

AFM-0026:R1 pinned `model::{AdrRecord, DomainDir, AdrId, ...,
parse_adr_id}` as the library surface but, like the trait-floor gap
AFM-0028 closed for `LoadError`, did not constrain field *visibility*.
Every field was `pub`: a consumer could construct an
invariant-violating value directly — a lowercase/empty `AdrId.prefix`,
a `number` above `9999`, or an `AdrRecord` with mutually inconsistent
parser-derived fields. `AdrId`'s rustdoc already documented these
invariants as "parser-produced values only" — an admission the type
could not defend them itself.

`adr-fmt` has not shipped to crates.io; this is the last point where
tightening visibility is pre-release, not breaking. Reconstruction
from untrusted input needs `Result`, not `Option`: `None` folds every
failure reason into one absence — error-is-not-a-negative-finding,
generalized to construction. New surface beyond AFM-0026:R1 needs its
own ADR under AFM-0026:R5; `adr-fmt` itself is current consumer
(pre-1.0 correctness ahead of the release that makes further change
breaking).

AFM-0029 supersession was rejected: only two types' visibility
changes, not the whole surface. In-place body edit is foreclosed by
the AFM-0028 precedent: Accepted ADRs amend via successor. This ADR
follows that precedent — Route A, amendment via successor, targeting
AFM-0026 directly.

## Decision

Amend AFM-0026 by tightening the field visibility of `AdrId` and
`AdrRecord` to `pub(crate)`, adding validated construction and read-only
accessors as the sole external API for those two types, and reaffirming
that the new surface inherits AFM-0028's trait-floor rule.

R1 [5]: `AdrId` and `AdrRecord` fields are `pub(crate)`, not `pub`. No
  external consumer (including `adr-srv`) can construct or field-access
  either type directly; both are true by construction now rather than
  documented by convention.

R2 [5]: The sole external `AdrId` constructors are `try_new(prefix,
  number) -> Result<Self, AdrIdError>` and `TryFrom<&str>`. Both
  validate the AFM-0026:R1-documented invariants and reject violations
  with `AdrIdError`. `parse_adr_id` / `parse_adr_id_from_filename_stem`
  keep their `Option` signatures unchanged (already-pinned surface).

R3 [5]: `AdrRecord` has no external constructor — `parse_domain` /
  `parse_stale` are the only routes to a live instance outside the
  crate. External reads are limited to accessors for the fields the
  current consumer (`adr-srv`) reads: `id`, `file_path`, `title`,
  `date`, `last_reviewed`, `tier`, `status`, `relationships`. Unread
  fields get no accessor here; adding one later needs a `pub fn`, not
  a further ADR.

R4 [5]: `AdrIdError` — the error type returned by R2's constructors —
  inherits the AFM-0028:R1 trait floor by construction
  (`core::fmt::Display`, `core::fmt::Debug`, `std::error::Error`) as a
  new type added to the AFM-0026:R1 surface under AFM-0026:R5. No
  separate ADR is needed to establish this per AFM-0028:R4.

R5 [5]: The trait surfaces, accessor signatures, and constructor
  signatures introduced by this ADR are part of the v0.1 semver
  contract per the extension of AFM-0026:R3. New accessors or trait
  impls may be added in minor versions; none introduced here may be
  removed or reshaped without a successor ADR.

## Consequences

+ becomes easier: no consumer can construct an `AdrId` or `AdrRecord`
  that violates AFM-0026:R1's documented invariants; the type system
  enforces what prose previously asserted, already true at the first
  crates.io release.
− becomes harder: reading an `AdrRecord` field beyond the R3 accessor
  set needs a new accessor; a genuinely new external constructor for
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
