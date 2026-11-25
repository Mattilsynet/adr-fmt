# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2026-11-25

### Changed

- Wire the AFM-0038 bounded warning-details proof
  (`tests/warning_factory.py`) into `cargo test` so the shipped
  restriction-surfacing behaviour actually runs instead of being orphaned.
- Root-anchor the `graphify-out/` gitignore entry per sf-sdlc so a graphify
  index anywhere under the tree stays untracked.
- Keep stray `.DS_Store` and scratch artifacts out of tracking via gitignore.

## [0.4.0] - 2026-11-25

First release with a recorded changelog; the entries describe the capabilities
this release provides, not a diff against an earlier tagged release.
Version 0.3.0 was not published (AFM-0040:R3).

### Added

- Read-only ADR template and link-integrity validator with five modes: default
  governance guidelines, `--lint`, `--refs`, `--context`, and `--tree`.
- Corpus discovery by walking up to the nearest suitable `adr-fmt.toml`,
  failing on present broken or inaccessible markers while genuinely absent
  markers and intentionally unfit readable configurations may ascend.
- Six-technique conventions in generated governance output per AFM-0039.

### Changed

- MSRV floor raised to Rust 1.100, with two recorded raises 1.98→1.99 and
  1.99→1.100 per AFM-0040.
- Retrieval exits nonzero while lint stays advisory exit 0 with truthful totals
  per AFM-0039 R2–R4.
- Context output preserves each rule's normative strength and ancestry stops at
  stale/Rejected/Deprecated/Superseded per AFM-0039 R5–R6.

[Unreleased]: https://github.com/Mattilsynet/adr-fmt/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/Mattilsynet/adr-fmt/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Mattilsynet/adr-fmt/releases/tag/v0.4.0
