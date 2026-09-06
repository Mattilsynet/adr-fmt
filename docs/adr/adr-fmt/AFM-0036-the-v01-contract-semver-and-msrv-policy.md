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
1.98.0 as a build toolchain and `Cargo.toml` declares no
`rust-version`, so consumers are promised no minimum compiler.

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

R4 [5]: While the version is 0.1.x every release MUST be non-breaking
  under R3, and additive change is permitted. A break requires the
  version to move to 0.2.0 and a successor ADR naming what broke, on
  the terms AFM-0029:R2 sets for recording it

R5 [5]: The channel `rust-toolchain.toml` pins is the MSRV floor.
  Publishing this crate MUST add a `Cargo.toml` `rust-version` equal
  to that channel; until then no MSRV is published and none is owed.
  Raising the floor is breaking under R3 and obliges R4's bump

## Consequences

The phrase has a referent. Each leaning ADR can cite R2 for what it
claims membership of and R3 for what breaking it means, instead of
asserting membership on its own authority. R3 costs the most: it makes
the rendered output and the exit codes contract surface, so a wording
change to the guidelines rendering is a 0.2.0 matter rather than a
patch — which is already how the byte-level golden pin treats those
bytes. R5 leaves the crate unpublished and owing no MSRV; adding
`rust-version` is the act that converts the build pin into a promise.
