# adr-fmt <tag>

`adr-fmt` is a read-only ADR template and link-integrity validator. It scans
an ADR corpus and reports template, reference and citation findings; it never
writes to the corpus.

This is the first published release: **0.4.0**. There was no published 0.3.0.
The MSRV floor is **1.100**, raised under AFM-0040.

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
[`scripts/adr-lint-gate.sh`](https://github.com/Mattilsynet/adr-fmt/blob/<tag>/scripts/adr-lint-gate.sh),
which parses the diagnostics header and separates over-threshold findings from
its no-verdict result.

Any lint count quoted must name the **binary revision** and the **corpus
state**, because the consumer's corpus is edited independently of the binary;
a count without both is not reproducible.

## Reading health scores

A health score is band-based: a size reduction can move a score backwards until
a band boundary is crossed, so a lower score does not by itself mean a
regression.
