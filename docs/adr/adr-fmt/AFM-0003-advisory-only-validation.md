# AFM-0003. Advisory-Only Validation With Exit-Code Semantics

Date: 2026-04-27
Last-reviewed: 2026-09-05
Tier: B
Status: Accepted

## Related

References: AFM-0001, GND-0004

## Context

Lint tools face a tension between strictness and adoption. Failing
builds on any warning pressures authors to suppress rather than fix.
ADR files are authored as prose — drafts are intentionally incomplete
and proposed ADRs may have placeholder relationships. Forcing zero
warnings before merge would discourage ADR creation. Two exit-code
strategies exist: non-zero on warnings (risks suppression) or zero
on warnings with non-zero only for infrastructure errors (risks
overlooked warnings without process discipline). Enforcement therefore
belongs to a wrapper that reads the tool's own summary line, so the
threshold is a project policy rather than a property of the binary.

## Decision

`adr-fmt` exits 0 for all lint findings and exits 1 only for
infrastructure errors. All advisory diagnostics — both rule
findings and parser-stage findings (AFM-0017) — emit warnings,
never errors.

R1 [5]: Return exit 0 for all lint completions and exit 1 only for
  infrastructure failures (missing config, unreadable directories,
  invalid configuration) reported via stderr in main outside the
  Diagnostic channel
R2 [5]: Emit every advisory finding (rule findings and parser-stage
  findings per AFM-0017) as Severity::Warning via
  Diagnostic::warning in adr-fmt/src/report.rs; the Severity enum
  exposes only the Warning variant for the advisory diagnostic stream
R3 [5]: Enforce the warning threshold outside `adr-fmt` in
  `scripts/adr-lint-gate.sh`, which parses the `## Diagnostics: N
  warning(s)` header on stdout, exits 1 above the threshold, and
  exits 2 when it cannot obtain that count; run it locally and in CI

## Consequences

Authors can write Draft ADRs with incomplete sections without being
blocked. Threshold enforcement lives in `scripts/adr-lint-gate.sh`,
which contributors run locally rather than discovering the policy
only in CI. The "exit 0 does not mean clean" semantics
must be documented. Future `--error-on-warning` flag is compatible
as a mode change. The model aligns with Rust conventions: `cargo
fmt` and `cargo clippy` default to non-blocking output.
