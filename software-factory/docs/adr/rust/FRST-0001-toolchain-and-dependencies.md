# FRST-0001. Toolchain and Dependencies

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FRST-0001

## Context

Rust builds depend on compiler, feature and dependency choices that can drift independently between environments.

## Decision

The optional Rust profile makes build inputs and quality gates explicit.

R1 [5]: Rust repositories MUST pin their supported toolchain and declare compatibility expectations; upgrades MUST include affected tests and compatibility evidence.
R2 [5]: Cargo dependency declarations MUST have one authoritative inventory, minimal justified features and a documented lockfile policy.
R3 [5]: Delivery MUST run formatting, tests and configured lint checks without warnings; repositories SHOULD use stable rustfmt defaults and Clippy pedantic.

## Consequences

Toolchain changes become deliberate without importing any upstream compiler version, workspace layout or CI provider.
