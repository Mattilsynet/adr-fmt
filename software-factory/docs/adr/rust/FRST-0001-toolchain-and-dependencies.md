# FRST-0001. Toolchain and Dependencies

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FRST-0001

## Context

Rust builds depend on compiler, feature and dependency choices that can drift independently between environments. This independent root governs reproducible build inputs and delivery checks, not a particular API design or CI provider; no parent decision supplies those choices.

## Decision

The optional Rust profile makes build inputs and quality gates explicit.

R1 [5]: Rust repositories MUST pin their supported toolchain and declare compatibility expectations; upgrades MUST include affected tests and compatibility evidence.
R2 [5]: Cargo dependency declarations MUST have one authoritative inventory, minimal justified features and a documented lockfile policy.
R3 [5]: Rust delivery MUST run formatting, tests and configured lint checks without warnings; repositories SHOULD use stable rustfmt defaults and Clippy pedantic.

## Consequences

+ becomes easier: reviewing deliberate compiler, feature and dependency changes.

− becomes harder: upgrades require compatibility evidence and maintaining configured checks.

risks/migration: a pinned compiler does not establish dependency safety; no upstream version, workspace layout or CI provider is imported.

Evidence: [SOURCES](../../../SOURCES.md#repository-families) records Rust lineage and exclusions. Review R1–R3 against an adopter's toolchain, manifest inventory, lockfile policy and check results; these local source mappings are not executed adopter build evidence.
