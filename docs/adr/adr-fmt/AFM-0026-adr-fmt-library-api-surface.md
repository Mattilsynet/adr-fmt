# AFM-0026. adr-fmt Library API Surface

Date: 2026-05-18
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0006, AFM-0017, AFM-0001, CHE-0030, SEC-0004, COM-0007, COM-0013

## Context

`adr-fmt` exposes a library beside its read-only binary. The first parent,
AFM-0006:R1, constrains the parser whose records this seam exposes; removing
that constraint would leave the pinned parsing approach unspecified. The
lower-tier parent is intentional, not grounds to retier either decision.
AFM-0017:R1 constrains parser diagnostics, and AFM-0001:R4 separates checks
from judgment. Foreign citations retain their original supporting role;
their current contents are not verified by this local corpus.

The intended `adr-srv` adapter needs parsed records and nameable errors,
not private rendering modules. Flat crate-root exports keep that boundary
small without promising a future crate split. AFM-0028:R1 supplies the
error trait floor; AFM-0035:R1 pins observable meaning; AFM-0036:R2–R4
governs compatibility across the current 0.3.x series. `Diagnostic` retains
six public fields; no accessor migration is scheduled by a version label.

## Decision

Pin the `adr-fmt` library API to a flat re-export set at the crate
root, with all underlying modules private (CHE-0030:R1), the binary's
CLI shape unchanged, and the library forbidden from
calling `std::process::exit`.

R1 [5]: The crate root exposes exactly these items via
  flat `pub use` per CHE-0030:R1; modules are private, and
  reorganisation is non-breaking:
  `config::{Config, LoadError, load_quiet, resolve_corpus_root, ResolveCorpusError}`,
  `containment::{ContainmentError, contained_join, contained_join_optional}`,
  `model::{AdrRecord, DomainDir, AdrId, AdrIdError, Tier, Status, Relationship, RelVerb, parse_adr_id}`,
  `parser::{parse_domain, parse_stale, ParseOutcome, ParseError}`,
  `report::{Diagnostic, Severity}`,
  and defined in `lib.rs`: `run`, `RunError` (R10).
  `config::load` is intentionally absent; adding it requires
  COM-0013:R1 justification.

R2 [5]: Modules `context`, `nav`, `output`, `refs`, `rules`,
  `guidelines`, and `index` are crate-private. They are implementation
  details of the binary's `run()` entry point and MUST NOT be named by
  external consumers. Internal restructuring of these modules —
  splitting, merging, renaming — is a non-breaking change for
  downstream crates and requires no ADR.

R3 [5]: The `report::Diagnostic` struct's public-field shape is part
  of the v0.1 contract: fields `severity`, `rule`, `file`, `line`,
  `message`, `internal` are semver-stable. New fields may be added
  in minor versions; existing fields MUST NOT be removed or reshaped.
   Migration to `#[non_exhaustive]` plus accessors requires a successor
   ADR under AFM-0036:R4; no release is scheduled for that migration.

R4 [5]: Library code MUST NOT call `std::process::exit`. Errors
  surface as `Result` to the caller; `src/main.rs` is
   the only authorised exit-code site; callers retain process ownership.

R5 [7]: The library MUST NOT widen what the binary's CLI promises;
   compatibility of that CLI follows AFM-0036:R2–R4. New public library items
  beyond the R1 set require their own ADR with current-consumer
  justification per COM-0013:R1. AFM-0006 (regex parsing) and
  AFM-0017 (P0xx namespace) further pin the shape of items already
  exposed.

R6 [5]: Variant field shape of public error types in the R1 set is
  v0.1-stable, the reading AFM-0028:R3 already assumes. New variants
  may be added in minor versions; removing or reshaping one requires
   a recorded amendment and version transition under AFM-0036:R4.

R7 [5]: R1 pins field shape transitively: a type reachable through a
  pinned item's public signature is v0.1-stable on R3's terms, since a
   consumer cannot use the pinned item without naming it.

R8 [5]: `ContainmentError::MetadataProbeFailed` MUST preserve the path
   segment and typed I/O error kind so callers can distinguish permission
   failure from other indeterminate filesystem probes.

R9 [7]: Items in the R1 set MUST NOT name a third-party crate's type or
  trait in a public signature, a trait bound, an `impl Trait` return, or
  field shape reachable per R7. Implementing such a trait for a local
  type is exempt. `tests/public_surface_coupling.rs` pins the couplings
  its syntactic walk reaches: `toml::Value` in
  `config::RuleConfig::params`, via `Config::rules`. Widening requires an
  ADR.

R10 [5]: `run` MUST return `Result<(), RunError>`, and `RunError` MUST
  implement Display, Debug and `std::error::Error` per AFM-0028:R4. Its
  pinned meaning per AFM-0035:R1: `run` returns to its caller and MUST
  NOT terminate the process, and `src/main.rs` alone maps the result
  onto the AFM-0003:R1 exit codes. Pinning `run` does not make R2's
  private modules nameable, and R7 transitivity stops at `RunError`.

## Consequences

+ becomes easier: consumers name the pinned exports without depending on
  private module organization.
− becomes harder: additional exports require current-consumer justification
  and compatibility review rather than ad-hoc exposure.
risks/migration: field and meaning changes follow AFM-0036:R4; no crate split
  or Diagnostic accessor migration is pre-authorized.

Evidence: `src/lib.rs:33–54` exposes the flat API;
`src/containment.rs:32–39` defines the typed probe error.
`tests/lib_smoke.rs` probes exports and process ownership. Verification and
scope exclusions are recorded under mission `adr-fmt-rjo8.3`; downstream
`adr-srv` execution and foreign citation entailment are not established here.
