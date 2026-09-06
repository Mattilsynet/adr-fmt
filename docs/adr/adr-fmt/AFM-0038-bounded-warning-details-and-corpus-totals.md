# AFM-0038. Bounded Warning Details and Corpus Totals

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0036, AFM-0003, AFM-0029

## Context

Large diagnostic streams make individual repairs difficult to identify.
Limiting the scan instead of the presentation would hide corpus health and
allow threshold gates to pass despite findings. Source identity must also
survive failed parsing and native paths whose display strings collide.
AFM-0036:R4 is the constraining parent: observable diagnostic-output changes
need a recorded version transition; AFM-0003:R1 keeps findings advisory.

## Decision

R1 [5]: Lint output MUST show detailed warnings for one offending document
  by default, while preserving whole-corpus scanning and the existing public
  library signatures and diagnostic fields

R2 [5]: The lint-only `--max-warning-docs N` option accepts bare unsigned
  decimal integers fitting usize; zero suppresses document detail only,
  and invalid, missing, signed or overflowing arguments are usage errors

R3 [5]: Select offending documents by native source-path order, including
  malformed sources; clean documents consume no slots and global diagnostics
  remain visible independently of the document limit

R4 [5]: Count every public diagnostic before detail suppression, preserve
  the existing header and clean bytes, and emit sorted nonzero rule-kind
  totals; internal diagnostics remain excluded from output and counts

R5 [5]: Preserve the duplicate-ID short circuit and explicitly disclose
  skipped rule checks; its totals cover emitted findings only, and neither
  absent detail nor successful exit establishes a clean corpus

R6 [5]: Bound selection metadata by observed diagnostics rather than the
  requested limit; the option imposes no byte or ingestion-memory bound,
  and configuration and directory diagnostics remain outside its budget

## Consequences

+ becomes easier: bounded document detail keeps total findings visible.
− becomes harder: consumers needing more detail must request a larger limit.
risks/migration: document count is not a byte, memory or deadline bound; successful
  lint exit is not a clean verdict.

This limited successor records the 0.2.0 diagnostic-output break under
AFM-0036:R4, not whole replacement; the policy remains in the 0.3.x series.
Evidence: `src/output.rs:131–210` counts public findings before suppression;
`src/lib.rs:354–373` discloses incomplete duplicate-ID validation. No absolute
resource or diagnostic-completeness guarantee follows for skipped rule checks.
