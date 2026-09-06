# AFM-0027. adr-fmt ↔ adr-srv Boundary

Date: 2026-05-18
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0026, AFM-0006, AFM-0017, AFM-0001, CHE-0029, CHE-0030, COM-0012, COM-0013

## Context

The `adr-srv` adapter projects parsed ADR records into service events.
AFM-0026:R1 is the first parent because its Accepted library surface defines
the types this boundary can consume; without that surface, the adapter
contract has no stable input. AFM-0026:R2 keeps implementation modules private.

Event mapping and scrape idempotency belong outside the read-only parser.
Otherwise service state and event dependencies would enter the governance
tool. AFM-0017:R1 constrains parser diagnostics; foreign architectural
citations remain supporting references, not locally checked authority.
This ADR specifies ownership without asserting the current implementation
state of the external service.

## Decision

Anchor the seam: dependency points one way, the pardosa bridge lives
on the `adr-srv` side, `Diagnostic` is re-projected not re-exported,
idempotency is `adr-srv`'s problem, and parser-shape changes invalidate
prior scrapes wholesale.

R1 [5]: `adr-srv` depends on `adr-fmt` (library); the reverse is
  forbidden. `adr-fmt` MUST NOT name `adr-srv` in any
   `[dependencies]`, `[dev-dependencies]`, or `cfg`-gated path.

R2 [5]: The pardosa bridge — the mapping from `adr_fmt::AdrRecord`
  (and friends re-exported per AFM-0026:R1) into pardosa-genome event
  envelopes — lives in `adr-srv`, not in `adr-fmt`. `adr-fmt` MUST
  NOT take any `pardosa-*` dependency, direct or transitive via
   workspace features.

R3 [5]: `adr-srv` MUST re-project `adr_fmt::Diagnostic` (per
  AFM-0026:R3) into its own API type — GraphQL, JSON, or otherwise.
  `adr-srv` may add fields to its projection but MUST NOT depend on
  `Diagnostic` internals beyond what AFM-0026:R3 pins. When
   AFM-0026:R3's stability posture evolves, `adr-srv` adapts on its side
  without `adr-fmt` knowing.

R4 [5]: Scrape idempotency is `adr-srv`'s responsibility. `adr-srv`
  computes a content hash (`body_hash` or equivalent) per
  `AdrRecord` and skips unchanged events on re-scrape. `adr-fmt`'s
  `parse_domain` and `parse_stale` remain pure walk-and-parse,
  stateless; no body-hash field is added to `AdrRecord` for
  `adr-srv`'s benefit per COM-0013:R1 (no speculative complexity
  without a current consumer in the inner crate).

R5 [7]: When `adr-fmt`'s parser shape changes — a new `P0xx` code per
  AFM-0017, an added `AdrRecord` field, a removed `Tier` variant, or
  any other change to `parse_domain` / `parse_stale` output —
  `adr-srv` MUST re-scrape from scratch. There is no migration
   contract between `adr-fmt` versions; compatibility and parser
   evolution follow AFM-0036:R2–R4.

R6 [5]: `adr-srv` MAY load a known marker directory with
   `adr_fmt::load_quiet` and `adr_fmt::resolve_corpus_root`, or pass
   a pre-resolved corpus root to parsing. These helpers MUST NOT be
   described as implementing the binary's ancestor-discovery loop.

## Consequences

+ becomes easier: parser consumers and service adapters have separate owners
  for input records, projections and scrape state.
− becomes harder: parser evolution requires a full downstream re-scrape;
  a service wanting ancestor discovery must supply that orchestration.
risks/migration: retain all supporting references despite T020; moving the
  Accepted API parent first reflects the actual dependency, not warning suppression.

Evidence: `src/lib.rs:33–54` keeps modules private and exports `Diagnostic`;
`src/config.rs:157–177,252–268` takes a known marker directory. Local checks
are recorded under `adr-fmt-rjo8.3`; external service integration is unverified.
