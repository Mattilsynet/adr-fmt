# AFM-0003. Advisory-Only Validation With Exit-Code Semantics

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0001, GND-0004

## Context

Draft ADRs can be incomplete without making lint execution fail. Separating
findings from execution failures lets a project set its own warning threshold.
AFM-0001:R4 is the first-parent constraint: mechanically enforceable policy
needs an executable check, here split between advisory detection and a wrapper
gate. GND-0004 is retained as a foreign citation, not independently verified
authority for these exit semantics.

## Decision

`adr-fmt --lint` exits 0 for findings; infrastructure failure exits 1.
Usage errors exit 2, and incomplete retrieval exits 1 (AFM-0039:R2).
All advisory diagnostics — both rule
findings and parser-stage findings (AFM-0017) — emit warnings,
never errors.

R1 [5]: Return exit 0 for all lint completions and exit 1 only for
  infrastructure failures (missing config, unreadable directories,
   invalid configuration) reported via stderr outside the
  Diagnostic channel
R2 [5]: Emit every advisory finding (rule findings and parser-stage
  findings per AFM-0017) as Severity::Warning via
   Diagnostic::warning in src/report.rs; the Severity enum
  exposes only the Warning variant for the advisory diagnostic stream
R3 [5]: Enforce the warning threshold outside `adr-fmt` in
  `scripts/adr-lint-gate.sh`, which parses the `## Diagnostics: N
  warning(s)` header on stdout, exits 1 above the threshold, and
  exits 2 when it cannot obtain that count; run it locally and in CI

## Consequences

- Easier: authors inspect incomplete drafts without a binary-level
  zero-warning requirement.
- Harder: automation must inspect the summary through the wrapper, not treat
  lint exit 0 as clean.
- Risks: thresholds tolerate findings; duplicate IDs prevent complete validation
  and produce gate exit 2 under AFM-0039:R3.

Source evidence: `src/main.rs:18–23` maps exits;
`src/lib.rs:342–373` renders advisory and incomplete lint;
`scripts/adr-lint-gate.sh:30–93` validates counts and enforces the threshold.
These mechanisms do not establish that warnings are harmless or measure
effects on author behavior.
