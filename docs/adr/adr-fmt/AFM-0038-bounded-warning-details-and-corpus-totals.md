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

## Decision

R1 [5]: Version 0.2.0 changes lint output to show detailed warnings for one
  offending document by default, while preserving whole-corpus scanning and
  the existing public library signatures and diagnostic fields

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

This is the limited successor recording the diagnostic-output break required
by AFM-0036, not a whole-ADR replacement under AFM-0029. Other contract
surfaces and the compiler floor remain in force. Consumers needing more
document details must request a larger limit; enforcement consumers retain
the truthful header instead of inferring health from visible bullets.
