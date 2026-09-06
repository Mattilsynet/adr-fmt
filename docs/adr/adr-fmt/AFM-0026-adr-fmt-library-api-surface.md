# AFM-0026. adr-fmt Library API Surface

Date: 2026-05-18
Last-reviewed: 2026-09-05
Tier: S
Status: Accepted

## Related

References: AFM-0006, AFM-0017, AFM-0001, CHE-0030, SEC-0004, COM-0007, COM-0013

## Context

`adr-fmt` ships as both a binary (the SSOT per AFM-0001) and a library
in the same crate. With Track 3.2 (`adr-srv`) imminent, the library
seam becomes a cross-crate contract and merits an explicit pin. The
surface is defined only by what `lib.rs` happens to expose; the oracle
summary at bd `adr-fmt-d7ao` enumerates the minimum set `adr-srv`
needs, items currently over-exposed, and the drift from CHE-0030. The
predecessor mission (bd `adr-fmt-mvtu`; commits `ebe791f` T2 lift,
`be0b552` Q2 trim) tightened the surface in-code; this ADR pins it.

Three pressures shape the decision. The binary's CLI is the surface
users already depend on and stays unchanged for v0.1; no ADR pins a
wider one, so the library widening past it would bind the crate to a
surface nothing governs. That constraint lives in R5 of this ADR.
SEC-0004:R3 and COM-0007:R4 prefer minimal default-private surfaces.
COM-0013:R1+R4 forbids speculative complexity and prefers the more
reversible design — flat `pub use` at the crate root is reversible
into a future `adr-fmt-core` split without consumer-side change.

`adr-srv` is the sole intended consumer. Pinning a small surface now
is cheaper than negotiating a wider one later.

The R1 set's membership follows from its consumer, not from what
`lib.rs` happens to expose. `adr-srv`'s scrape pipeline projects
`AdrRecord`s into the `AdrIngested` event payload and names
`model::{Status, Relationship, RelVerb}` directly, so those sit at the
crate root rather than behind a private path. `ResolveCorpusError`,
`ParseError` and `AdrIdError` are the `Err` of items already pinned —
`config::resolve_corpus_root`, `parser::parse_domain`,
`parser::parse_stale`, and `AdrId::try_new` with its `TryFrom<&str>`
impl — and a consumer cannot call the pinned API without naming them,
so pinning them records existing reality rather than widening the
surface (AFM-0028:R4 error-type inheritance). `adr-srv` calling those
functions is the current-consumer justification COM-0013:R1 requires
for every member of the set. `index` is crate-private on R2's terms:
it is an implementation detail of the binary's `run()` entry point.

R1 and R7 pin which items and fields are stable but are silent on
whose types they are, so R9 governs semver coupling to third-party
crates — `clap`, `regex`, `serde`, `toml` — in that same surface.

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
  Migration to `#[non_exhaustive]` plus accessors is deferred to v0.2
  and requires a successor ADR. `adr-srv` is the only known consumer
  and simplicity dominates.

R4 [5]: Library code MUST NOT call `std::process::exit`. Errors
  surface as `Result` to the caller; `src/main.rs` is
  the only authorised exit-code site. Pins the T2 lift landed in
  commit `ebe791f` against regression and reflects SEC-0004:R2
  (authority passed explicitly, never via global process state).

R5 [7]: The library MUST NOT widen what the binary's CLI promises;
  that CLI shape is unchanged for v0.1. New public library items
  beyond the R1 set require their own ADR with current-consumer
  justification per COM-0013:R1. AFM-0006 (regex parsing) and
  AFM-0017 (P0xx namespace) further pin the shape of items already
  exposed.

R6 [5]: Variant field shape of public error types in the R1 set is
  v0.1-stable, the reading AFM-0028:R3 already assumes. New variants
  may be added in minor versions; removing or reshaping one requires
  an in-place amendment naming the break per AFM-0029:R2. Recorded
  break: `ContainmentError::CanonicalizeFailed`, removed in commit
  `8a34c4e`.

R7 [5]: R1 pins field shape transitively: a type reachable through a
  pinned item's public signature is v0.1-stable on R3's terms, since a
  consumer cannot use the pinned item without naming it. Recorded
  break: `config::DomainConfig`, reachable via `Config::domains`, lost
  inert field `multi_root_rationale` in commit `0642ad1`.

R8 [5]: Second break recorded under R6: `ContainmentError::MetadataFailed`
  carried a stringly `reason`, leaving permission failure
  indistinguishable from transient I/O error. Commit `b139537` removes
  it in favour of the typed `MetadataProbeFailed { segment, kind }`;
  additive half in `01aaa7a`.

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

+ becomes easier: Track 3.2 (`adr-srv`) depends on a pinned, documented
  library surface without spelunking through `lib.rs`. Internal
  reorganisation of the six private modules no longer risks breaking
  downstream crates. The CHE-0030 doctrinal drift recorded in oracle
  bd `adr-fmt-d7ao` (T1) is resolved.
− becomes harder: any future need for an item outside the R1 set —
  `config::load`, `nav::ChildEntry`, `rules::run_all`, deeper
  `context` access — requires a follow-up ADR rather than ad-hoc
  exposure. Speculative widening is forbidden.
risks/migration: reversibility per COM-0013:R4 — the current `lib+bin`
  arrangement can later be split into `adr-fmt-core` + `adr-fmt`
  without surface change for consumers, since the surface is at the
  crate root via flat `pub use`. This ADR does not pre-authorise that
  split; re-evaluate when a second non-`adr-srv` consumer appears.
