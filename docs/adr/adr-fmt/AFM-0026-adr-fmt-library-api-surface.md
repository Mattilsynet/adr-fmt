# AFM-0026. adr-fmt Library API Surface

Date: 2026-05-18
Last-reviewed: 2026-09-02
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

Three pressures shape the decision. AFM-0001:R1 freezes the binary CLI
for v0.1; the library MUST NOT widen what the binary promises.
SEC-0004:R3 and COM-0007:R4 prefer minimal default-private surfaces.
COM-0013:R1+R4 forbids speculative complexity and prefers the more
reversible design — flat `pub use` at the crate root is reversible
into a future `adr-fmt-core` split without consumer-side change.

`adr-srv` is the sole intended consumer. Pinning a small surface now
is cheaper than negotiating a wider one later.

Amendment 2026-05-19 (Phase 2 v2 M1.3): R1 broadened to add
`model::{Status, Relationship, RelVerb}`. The `adr-srv` scrape
pipeline projects `AdrRecord`s into the `AdrIngested` event payload
and names these three types directly. They were already public on
`model`; the amendment moves them into the pinned crate-root re-export
set so `adr-srv` does not name a private path. No new types.

Amendment 2026-08-13 (M1 cleanup): R1 broadened to add
`config::ResolveCorpusError` and `parser::ParseError`; R2 extended
to name the `index` module in its crate-private enumeration. Both
error types are the `Err` of `Result`s returned by functions already
pinned in R1 (`config::resolve_corpus_root`, `parser::parse_domain`,
`parser::parse_stale`); a consumer cannot call the pinned API without
naming them, so this pins existing reality rather than widening the
surface (per AFM-0029:R2 in-place amendment, AFM-0028:R4 error-type
inheritance). `index` is a private module of the binary's `run()`
entry point, matching the R2 enumeration's existing members. Current
consumer: `adr-srv`, via the three functions above.

Amendment 2026-09-02 (cluster-6 finding #8): R6 added to state the
v0.1 stability of error-variant field shape explicitly — previously
only implied by AFM-0028:R3's back-reference to R3 — and to record one
break. `containment::ContainmentError::CanonicalizeFailed
{ segment, reason }` erased which operand failed and flattened
`std::io::ErrorKind` into arbitrary text, so a consumer could not
distinguish `NotFound` from an indeterminate failure. Commit `8a34c4e`
replaces it with `RootCanonicalizeFailed { segment, kind }` and
`TargetCanonicalizeFailed { segment, kind }`, both carrying a typed
`std::io::ErrorKind` (additive half landed in `e77ee78`). The removed
variant had no consumer outside this crate. Recorded in place per
AFM-0029:R2 — no Supersedes edge, no successor ADR, since this amends
one rule rather than replacing the ADR.

Amendment 2026-09-02 (SM-05 review finding N2): R7 added to state that
R1's "exactly these items" pins the reachable field shape
transitively, not only the named items — a consumer cannot use pinned
`Config` without naming `DomainConfig`, reached through the public
`Config::domains: Vec<DomainConfig>` — and to record one break.
`config::DomainConfig::multi_root_rationale` was public, parsed, and
inert: the warning it promised was never wired and no code read it.
Commit `0642ad1` removes it. The TOML schema is unaffected (no
`deny_unknown_fields`; no corpus `adr-fmt.toml` sets the key), but
removing a public field of a reachable type is a Rust source break for
struct literals and field access, so it is recorded here. Recorded in
place per AFM-0029:R2 — no Supersedes edge, no successor ADR.

Amendment 2026-09-02 (SM-06, opportunistic): R8 added to record a
second break under R6, which already governs it — R6 cannot absorb the
record without exceeding T016's 60-word limit.
`containment::ContainmentError::MetadataFailed` carried a stringly
`reason`, so a caller could not distinguish permission failure from a
transient I/O error at the type level — the same defect R6's first
recorded break named on the canonicalize path. Commit `b139537`
removes it in favour of `MetadataProbeFailed`, which carries
`std::io::ErrorKind`; the additive half landed in `01aaa7a`. Display
still names only the relative segment, per AFM-0028:R2. Recorded in
place per AFM-0029:R2 — no Supersedes edge, no successor ADR.

## Decision

Pin the `adr-fmt` library API to a flat re-export set at the crate
root, with all underlying modules private (CHE-0030:R1), the binary's
CLI shape unchanged (AFM-0001:R1), and the library forbidden from
calling `std::process::exit`.

R1 [5]: The library exposes exactly these items at the crate root via
  flat `pub use` per CHE-0030:R1; underlying modules are private, and
  internal reorganisation is non-breaking:
  `config::{Config, LoadError, load_quiet, resolve_corpus_root, ResolveCorpusError}`,
  `containment::{ContainmentError, contained_join, contained_join_optional}`,
  `model::{AdrRecord, DomainDir, AdrId, Tier, Status, Relationship, RelVerb, parse_adr_id}`,
  `parser::{parse_domain, parse_stale, ParseOutcome, ParseError}`,
  `report::{Diagnostic, Severity}`.
  `config::load` is intentionally absent; adding it requires a
  current-consumer justification per COM-0013:R1.

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

R5 [7]: The library MUST NOT widen what the binary's CLI promises per
  AFM-0001:R1 (frozen for v0.1). New public library items beyond the
  R1 set require their own ADR with current-consumer justification
  per COM-0013:R1. AFM-0006 (regex parsing) and AFM-0017 (P0xx
  namespace) further pin the shape of items already exposed.

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
