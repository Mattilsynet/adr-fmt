# AFM corpus guide

This corpus records the architecture of `adr-fmt` itself. The repository-root
[`adr-fmt.toml`](../../adr-fmt.toml) registers one domain, AFM, mapped to the
`adr-fmt` crate. No foundation domains are configured; the configurable
foundation capability remains available.

## Layout and authority

- [`adr-fmt/`](adr-fmt/) contains 29 Accepted ADRs.
- [`stale/`](stale/) contains 10 retired stubs. Their status and retirement
  sections identify disposition; git history preserves the full decisions.
- The [repository guide](../../AGENTS.md#governance) documents local checks.
- The [software-factory corpus](../../software-factory/README.md) has its own
  nested configuration; it is separate from this AFM corpus.

These counts describe the 39 ADRs present on 2026-09-06. References to other
corpora do not register those domains here; link-integrity scope follows
[AFM-0008](adr-fmt/AFM-0008-domain-scoped-prefix-naming.md).

## Discover the format and applicable decisions

Run these commands from the repository root to use the current checkout
rather than a potentially older installed binary:

```bash
cargo run -q --locked
cargo run -q --locked -- --tree AFM
cargo run -q --locked -- --context adr-fmt
cargo run -q --locked -- --refs AFM-0015
```

The no-flag command prints generated governance guidelines, including the
ADR structure and rule catalogue. There is no standalone template to copy:
[AFM-0004](adr-fmt/AFM-0004-template-defined-in-code.md) keeps the format
definition in code. This guide provides navigation, not another specification.

The [0.3.0 migration guide](../adr-format-proposals.md) points to the adopted
six-technique guidance. The [per-document matrix](six-technique-review.md)
records completed local author/source-review migration for all 29 active AFM
decisions, ten retired stubs and 24 factory decisions. Independent optional
factory approval is recorded in `adr-fmt-t8g9`; final boundary evidence is
tracked in `adr-fmt-rjo8.18` for commander review and acceptance. The matrix
records evidence limits rather than claiming adopter compliance.

`--tree` shows structural parents; the first `References:` target is the
parent, not an arbitrary ordering convenience
([AFM-0020](adr-fmt/AFM-0020-parent-edge-tree-model.md)). `--context adr-fmt`
selects decision rules for the crate; `--refs` shows inbound references.
For tier classification use
[AFM-0011](adr-fmt/AFM-0011-meadows-aligned-tier-classification.md), not warning
suppression as a reason to retier. For in-place amendments and whole-ADR
supersession use [AFM-0029](adr-fmt/AFM-0029-adrs-are-atomic.md); retired stub
structure is defined by [AFM-0022](adr-fmt/AFM-0022-stale-archive-stub-policy.md).

## Check the corpus

```bash
cargo run -q --locked -- --lint --max-warning-docs 10
cargo test --locked --test adr_citation_existence --test adr_message_citations
env PATH="$PWD/target/debug:$PATH" scripts/adr-lint-gate.sh
```

Lint is advisory: exit 0 means analysis completed, not zero findings
([AFM-0003](adr-fmt/AFM-0003-advisory-only-validation.md)). Inspect the
`## Diagnostics` header and totals by kind; the detail limit does not limit
the total findings. The local gate enforces a default threshold of 8 and
distinguishes over-threshold findings (exit 1) from no verdict (exit 2).
Its default producer is `cargo run -q --locked -- --lint`.

The measured corpus has 3 warnings across 39 ADRs on 2026-09-06: L016 × 1
and T020 × 2. These remaining tier/parent and reference
count advisories warrant review, not automatic governance changes. The gate
threshold remains 8. Citation tests check source citations against live ADR
rules; they do not replace corpus lint or the full Rust test suite.

AFM-0026 retains its parser parent and supporting references; AFM-0027
retains eight supporting references with the Accepted API contract first.
The three remaining advisories are disclosed, not suppressed. The two
oversized Context sections were shortened by removing stale claims.
