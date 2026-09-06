# AFM-0036. The v0.1 Contract, Semver and MSRV Policy

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0026, AFM-0028, AFM-0032, AFM-0035, AFM-0029, AFM-0001

## Context

Four ADRs pin items as part of "the v0.1 contract" — AFM-0026:R3,
AFM-0028, AFM-0032:R5 and AFM-0035 — and no ADR says what the phrase
denotes. Its content is the union of whatever those rules happen to
pin, so a surface nobody thought to pin sits outside it by accident
rather than by decision. The binary's CLI, its exit codes and its
rendered output are in that position: AFM-0026:R5 forbids widening
past a CLI shape that nothing pins. `rust-toolchain.toml` pins channel
1.98.0 and `Cargo.toml` declares `rust-version = "1.98"`: one compiler
floor stated twice, with nothing requiring the two to agree.

## Decision

Give the phrase a referent: name the surfaces it spans, what breaks
them, and how the version and the toolchain floor may move.

R1 [5]: "The v0.1 contract" denotes exactly the promises R2
  enumerates. An ADR asserting that an item is, or is not, part of the
  v0.1 contract MUST resolve that claim through R2; membership is not
  established by an item merely being public

R2 [5]: The contract spans four surfaces: the library items `lib.rs`
  re-exports, per AFM-0026:R1; the binary's CLI, its flags and
  arguments and their spellings; process exit codes and the bytes of
  default-mode, guidelines and diagnostic output; and the
  `adr-fmt.toml` keys AFM-0001:R2 owns

R3 [5]: A change is breaking when a conforming consumer can observe
  it: removal or reshape of an R2 item, and equally a changed
  observable result at an unchanged shape — a different exit code,
  different rendered bytes, a previously accepted config key now
  rejected. AFM-0035:R1 is this rule applied to accessors

R4 [5]: Within each 0.y series every release MUST be non-breaking
  under R3; additive change is permitted. A break moves the version
  to 0.(y+1).0 and requires a successor ADR naming what broke, on
  AFM-0029:R2's recording terms. The current series is 0.2.x;
  AFM-0038 records its diagnostic-output contract

R5 [5]: The `rust-toolchain.toml` channel and the `Cargo.toml`
  `rust-version` state one MSRV floor and MUST be equal; both are
  1.98. Raising that floor is breaking under R3 and obliges R4's
  version bump

## Consequences

The phrase has a referent. Each leaning ADR can cite R2 for what it
claims membership of and R3 for what breaking it means, instead of
asserting membership on its own authority. R3 costs the most: it makes
the rendered output and the exit codes contract surface, so a wording
change to the guidelines rendering requires the next minor series rather than a
patch — which is already how the byte-level golden pin treats those
bytes. R5 binds the build pin and the published floor to one another,
so raising the toolchain channel is a consumer-visible break rather
than a build detail.
