# Six-technique adoption and 0.3.0 migration

Date: 2026-09-06. The six techniques are adopted in generated author/reviewer
guidance under AFM-0039:R7. This file is a migration guide, not a second format
specification. Obtain current authority with `cargo run -q --locked` from the
repository root. Generated governance and setup output include executable
independent-corpus examples; `tests/guidelines_parity.rs` verifies their lint
and retrieval behavior. No standalone canonical template is maintained.

Local corpus author/source-review migration covers all 29 active AFM documents,
ten retired stubs and 24 factory decisions. The
[per-document review matrix](adr/six-technique-review.md) records individual
assessments, repairs and evidence limits. AFM and factory slices have
independent approval, including the twelve optional factory repairs in
`adr-fmt-t8g9`. The generated tier-prose correction and final boundary evidence
are tracked in `adr-fmt-rjo8.18`; their review and decision-owner acceptance
remain separate. This is completed local
author review, not package completion or adopter compliance inferred from lint.

## Apply the generated conventions

| Technique | Author migration | Review boundary |
|---|---|---|
| Standalone obligations | Keep action, object, scope and conditions in each extracted rule | Meaning and MUST/SHOULD/MAY strength require reading, not keyword counting |
| Justified first parent | Read the actual constraining rule before ordering References; explain why it constrains the decision | A root needs a justified absence of parent, not a fabricated reference |
| Crate-scoped retrieval | Use configured crate selectors, not copied rule bundles | Check positive and negative scope membership |
| Concise rationale and tradeoffs | Replace obsolete history with present rationale and explicit easier/harder/risks consequences | Do not retier or delete supporting citations to suppress advisories |
| Inspectable evidence | Link explanatory prose to source and a dated execution/review record | A command is a plan until an observed exit is recorded; existence is not entailment |
| Lifecycle and truthful totals | Amend narrow changes in place; retire whole replacements as stubs | Acceptance is a decision, not a lint exit code |

These are navigation summaries of generated authority, not new grammar.
`RN [L]: text`, two-space continuations, metadata, three relationship verbs,
and configuration keys remain unchanged. An extra H2 inside Decision ends
rule extraction; keep supporting evidence outside tagged rules and do not
interrupt them. Syntax checks do not establish semantic precision, parent
entailment, evidence freshness, acceptance, agent effectiveness or token savings.

## Observable 0.3.0 changes

AFM-0036:R2–R4 defines compatibility; AFM-0039 records this transition.

| Surface | Change | Consumer action |
|---|---|---|
| Discovery | Present directories, dangling symlinks and inaccessible markers fail rather than silently selecting an ancestor | Repair the intended marker; assert selected corpus identity |
| Refs/context/tree | Parser findings or duplicate ADR IDs cause exit 1, an incomplete diagnostic on stderr, and no authoritative stdout | Run lint, repair the source and retry; do not consume a partial answer |
| Lint gate | Duplicate-ID incomplete validation gives no-verdict exit 2 regardless of threshold | Do not equate a small emitted count with a complete scan |
| T016 messages | Duplicate rule IDs are distinguished from missing IDs; overflowing numeric layer tags retain source spelling | Update exact diagnostic pins, not grammar or public layer interpretation |
| Context preamble | No unconditional “without exception” mandate | Preserve each rule's strength and conditions in consuming prompts |
| Context ancestry | Terminal, stale and unknown-status nodes cannot anchor live ancestry; Draft/Proposed remain waypoints | Expect eligible descendants without a live root in Unclaimed, with all their rules retained |
| Generated output | Governance/setup bytes publish all six conventions and coherent examples | Update golden consumers deliberately; use no arguments, not a nonexistent `--guidelines` flag |
| Package identity | Cargo package version is 0.3.0 | Update version-sensitive consumers; the compiler floor stays 1.98 |

Unchanged: lint findings remain advisory exit 0; CLI usage errors exit 2;
public exports, diagnostic fields, parsed-rule compatibility representation,
configuration schema and rule grammar are not redesigned. All decision rules
of eligible non-stale Accepted ADRs are retained irrespective of layer.
Genuinely absent markers may ascend; intentionally unfit readable configurations
retain nearest-fit fallback. Scope completeness remains a review obligation.

## Verify the two corpora separately

From the repository root:

```sh
cargo build --locked
cargo run -q --locked -- --context adr-fmt
cargo run -q --locked -- --tree AFM
cargo run -q --locked -- --lint --max-warning-docs 10
scripts/adr-lint-gate.sh
cargo test --locked --test adr_citation_existence --test adr_message_citations --test guidelines_parity
python3 software-factory/test_verify.py
python3 software-factory/verify.py
```

The factory verifier checks its independent 24-ADR corpus, seven crate
selectors and a copied/remapped adopter corpus. Passing it establishes
mechanical compatibility, not completion of per-document six-technique review.
The root gate threshold remains 8; consult the measured totals in the
[corpus guide](adr/README.md). Hidden details do not reduce totals. Duplicate
IDs short-circuit rule checks, so emitted totals are not a complete verdict.

AFM-0029:R2 governs partial amendments and review dates; AFM-0022:R1–R2
governs retired stubs. No clause-level supersession or `Amended` status is
introduced. Root decisions do not need invented parents; still-active decisions
do not need artificial retirement. Foreign supporting citations are not imported
or erased merely because local link checks cannot resolve them.

## Resource and evidence limits

The invocation remains synchronous and finite, with owned parsed records and
rendered strings. Memory scales with input bytes, records and edges; there is
no absolute byte/item/deadline ceiling. Repeated parent walks, tree recursion,
output growth, blocked filesystem calls and concurrent filesystem races remain
explicit gaps. Warning-detail limits are not memory bounds. No hosted aggregate,
allocator, dependency, kernel-memory or performance guarantee is inferred.

Source and execution pointers live in the review matrix and mission
`adr-fmt-rjo8.3`. Prior runtime/generated checkpoints were separately reviewed;
the optional factory checkpoint has independent approval in `adr-fmt-t8g9`.
No external `adr-srv` build or foreign
corpus entailment review was performed, and no measurements are invented.
