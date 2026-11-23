# adr-fmt

`adr-fmt` is a read-only ADR template and link-integrity validator. It scans
an ADR corpus and reports template, reference and citation findings; it never
writes to the corpus.

This release requires Rust **1.100** or newer (AFM-0036:R5; the current floor
is recorded there).

## Exit contract

| Exit | Meaning |
|---|---|
| `0` | analysis complete, including advisory warnings |
| `1` | infrastructure failure, or retrieval that cannot establish completeness |
| `2` | CLI usage error |

A warning never fails the process. `--lint`, `--refs`, `--context` and `--tree`
inherit this contract, so exit `0` with warnings is a complete analysis, not a
clean corpus.

## Lint counts are advisory

Lint counts are advisory; threshold enforcement lives in
`scripts/adr-lint-gate.sh`, which parses the diagnostics header and separates
over-threshold findings from its no-verdict result.

Any lint count quoted must name the **binary revision** and the **corpus
state**, because the consumer's corpus is edited independently of the binary;
a count without both is not reproducible.
