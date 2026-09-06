# AFM-0039. Trustworthy Discovery and Retrieval

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0036, AFM-0001, AFM-0038

## Context

Discovery must not mistake an unusable marker for absence and select an
ancestor corpus. Retrieval cannot establish complete references or rules
when parsing omitted source material. AFM-0036:R4 requires a new minor
series for these observable exit and diagnostic changes, making it the
constraining first parent rather than a general design citation.

## Decision

R1 [5]: Version 0.3.0 MUST reject present nonfile, dangling or inaccessible
  discovery markers instead of selecting an ancestor; genuinely absent
  markers and intentionally unfit readable configurations retain nearest-fit
  discovery under AFM-0001:R7 and AFM-0001:R8

R2 [5]: CLI refs, context and tree retrieval MUST exit nonzero with an
  explicit incomplete diagnostic and no authoritative stdout when corpus
  parsing emits findings or duplicate ADR identifiers prevent indexing

R3 [5]: Lint MUST retain advisory exit zero and truthful diagnostic totals;
  the repository lint gate MUST return no-verdict exit two when duplicate
  identifiers prevent complete validation, regardless of its warning threshold

R4 [5]: T016 MUST distinguish duplicate rule numbers from missing numbers
  and report overflowing numeric layer tags using their source spelling;
  parsed rules and public layer values retain their compatibility representation

R5 [5]: Context output MUST preserve each extracted rule's normative strength
  and conditions, without an unconditional mandate; every decision rule from
  each eligible non-stale Accepted ADR MUST be emitted regardless of layer

R6 [5]: Context ancestry MUST stop at stale, Rejected, Deprecated, Superseded
  or unknown-status nodes, including roots; Draft and Proposed nodes remain
  advisory waypoints, and eligible descendants without live roots MUST retain
  every rule in the Unclaimed group

R7 [5]: Version 0.3.0 generated governance and setup output MUST adopt standalone
  obligations, justified parents and exact citations, crate retrieval, concise
  rationale and tradeoffs, inspectable evidence, and explicit lifecycle with
  truthful totals as author/reviewer guidance, not new grammar or semantic checks

## Consequences

+ becomes easier: identifying a wrong corpus selection or incomplete answer
  before treating the result as authority.

− becomes harder: consumers must handle retrieval failures for corpora they
  previously consumed partially. Run lint to locate and repair findings.

This limited successor replaces no whole ADR. Default-mode and setup bytes
publish six conventions and examples. Reviewers judge scope, entailment,
freshness and acceptance; deterministic checks establish syntax, not merit.
risks/migration: input-scaled memory, filesystem deadlines and concurrent filesystem races
remain outside the guarantee; no absolute resource bound is claimed.

Source evidence: `src/lib.rs:282–381,484–574` guards retrieval and discovery;
`src/context.rs:115–244,337–369` preserves eligible rules;
`src/rules/template.rs:826–874` distinguishes numeric findings;
`src/guidelines.rs:133–178` publishes the six conventions. Source inspection
does not establish external consumer readiness or semantic authoring quality.
