---
name: adr-fmt
description: Use when running adr-fmt lint, investigating ADR warnings, or checking corpus health; explains bounded details and truthful totals.
---

# adr-fmt

Run inside the intended corpus: configuration discovery walks upward to
`adr-fmt.toml`. A nested corpus has independent configuration.

```bash
adr-fmt --lint
adr-fmt --lint --max-warning-docs 5
adr-fmt --lint --max-warning-docs 0
adr-fmt --tree
adr-fmt --refs AFM-0038
adr-fmt --context adr-fmt
```

The 0.2.0 default shows all warnings for one offending source document in
native path order. Zero hides document detail only; global configuration and
directory warnings stay visible. Malformed documents consume slots; clean
files do not. N is a bare unsigned decimal usize and requires `--lint`.

Always inspect `## Diagnostics: N warning(s) across M ADR(s)` and the
nonzero totals by rule kind. Counts include hidden documents and exclude
internal findings; M counts successfully parsed ADRs. Never derive counts
from detailed bullets. Exit zero means advisory completion, not clean.
Duplicate IDs short-circuit rule checks and produce an explicit incomplete
validation notice: totals then cover only parser and duplicate findings.

Use `scripts/adr-lint-gate.sh` for this repository's threshold policy. It
distinguishes over-threshold findings (exit 1) from no verdict (exit 2).
For development verification, rebuild and put `target/debug` first on PATH;
do not mistake an older installed binary for the changed checkout.

The detail cap is not a byte or memory cap: scanning, globals and messages
are not bounded by N. Preserve the intended corpus and prior working edits.
Restart opencode after installing this skill so its discovery is refreshed.
