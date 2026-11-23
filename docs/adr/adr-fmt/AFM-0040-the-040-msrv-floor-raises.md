# AFM-0040. The 0.4.0 MSRV Floor Raises

Date: 2026-11-23
Last-reviewed: 2026-11-23
Tier: B
Status: Accepted

## Related

References: AFM-0036, AFM-0029

## Context

AFM-0036:R5 makes raising the MSRV floor breaking under R3 and obliges R4's
version bump with a successor ADR naming what broke, recorded on AFM-0029:R2's
terms. The floor moved twice without that record: 1.98 to 1.99, then 1.99 to
1.100. AFM-0036:R4 is the constraining parent rather than a general citation.

## Decision

R1 [5]: The MSRV floor raised from 1.98 to 1.99, an observable compiler
  requirement change that is breaking under AFM-0036:R3

R2 [5]: The MSRV floor raised from 1.99 to 1.100, a second breaking change
  under AFM-0036:R3

R3 [5]: Version 0.4.0 records both raises; by the maintainer's ruling it is
  the first published version and 0.3.0 was not published. Registry state was
  not established from this repository

R4 [5]: AFM-0039 binds its requirements to Version 0.3.0, which the
  maintainer ruled unpublished; those requirements remain in force and apply
  to the 0.4.0 behaviour

## Consequences

+ becomes easier: one published version states the compiler floor truthfully.

− becomes harder: consumers holding a compiler below 1.100 cannot build this
  release.

This limited successor replaces no whole ADR; AFM-0036 remains in force.
risks/migration: local manifest and toolchain pins establish the floor this
repository requires, not the compiler a consumer has installed.

Evidence: `Cargo.toml:1–7` declares 0.4.0 and Rust 1.100;
`rust-toolchain.toml:1–4` pins 1.100.0. Commits 86b6c51 and 5cc7861 raised
1.98 to 1.99, toolchain selectors then the manifest; 4fb7777 raised 1.99 to
1.100. Publication and downstream builds are not established by these local
files.
