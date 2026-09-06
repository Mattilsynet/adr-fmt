# AFM-0035. Accessor Meaning Is Part Of The Pinned Contract

Date: 2026-09-05
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0032, AFM-0026, AFM-0027

## Context

AFM-0032:R5 is the constraining parent because it pins the accessor shapes
whose meaning this decision protects. An unchanged `Option<&str>` signature
can still change provenance or turn a previously retained value into None.
AFM-0033:R4 supplies a concrete boundary: date validity is separate from the
parser-supplied value. AFM-0036:R3 applies this compatibility principle across
the current contract; external `adr-srv` behavior is not verified here.

## Decision

Extend AFM-0032:R5 so that the observable meaning of a pinned item is
pinned alongside its signature.

R1 [5]: A change to what a pinned accessor returns for an input it
  already accepts IS a breaking change, whether or not the signature
  moves, and MUST be treated exactly as AFM-0032:R5 treats a reshape —
  it requires a successor ADR

R2 [5]: R1 covers the value, its emptiness, and its provenance. A
  pinned accessor that returned the source text MUST NOT begin
  returning a normalised, defaulted or derived form, and one that
  returned `Some` for an input MUST NOT begin returning `None` for it

R3 [5]: New internal knowledge about a pinned field MUST reach callers
  through a new accessor rather than by re-interpreting an existing
  one. Where that knowledge is crate-internal it MUST use a
  crate-private accessor, per the AFM-0033:R4 precedent

R4 [5]: A change claiming to preserve meaning MUST carry a test that
  pins the accessor's output across the inputs whose interpretation
  moved, so the claim is discharged by the suite rather than by review

## Consequences

+ becomes easier: reviewers have a named remedy for meaning changes: preserve
  the accessor and expose new knowledge separately.
− becomes harder: additional accessors and characterization tests need maintenance.
risks/migration: tests cover their selected inputs, not every downstream consumer.
  AFM-0026:R3 schedules no Diagnostic accessor migration; the series is 0.3.x.

Evidence: `src/model.rs:414–440,2244–2311` preserves supplied date values while
testing separate validity outcomes. `src/parser.rs:569–581` defines the earlier
trimming/absence boundary. No new normalization or version-driven migration is
authorized by this review.
