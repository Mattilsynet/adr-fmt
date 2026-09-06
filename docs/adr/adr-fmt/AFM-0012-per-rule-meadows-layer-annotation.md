# AFM-0012. Per-Rule Meadows Layer Annotation

Date: 2026-04-28
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0011, GND-0005, GND-0008

## Context

AFM-0011:R3 supplies the tier mapping used to interpret each rule's layer,
so it is the constraining parent. A rule's intervention can differ from its
ADR's overall tier; inline annotation preserves that distinction without a
second metadata table. GND-0005 and GND-0008 are retained foreign citations,
not locally verified entailment evidence.

## Decision

Per-rule Meadows layer annotations classify each tagged rule by
the type of systemic intervention it represents, independent of
the ADR's overall tier.

R1 [5]: Write each tagged rule in `RN [L]: text` format where N is
  the sequential rule ID and L is the Meadows leverage layer (1-12)
R2 [5]: Select rule ID, layer, and text in a single pass using
  the parser regex `^R(\d+)\s*\[(\d+)\]:\s*(.+)` — apply to every
  tagged rule in every `## Decision` section
R3 [5]: Apply T016 to validate layer range 1–12 for all statuses
  and permit Draft and Proposed ADRs to carry tagged rules without
  exemption from the format requirement
R4 [7]: Hold tension between an ADR's tier and its rules'
  Meadows layers as a derivable property of the per-rule annotation;
  a T019 violation fires iff `layer_to_tier(rule.layer).rank() <
  adr_tier.rank()` (rule operates at higher leverage than the ADR
  tier warrants — asymmetric bound); equal or lower leverage passes
  silently; the layer annotation is the load-bearing input for any
   tier-tension diagnostic in `--lint`
R5 [6]: Render rules with layer suffix in `--context` output using
  the global identifier format `[PREFIX-NNNN:RN:LN]` — apply to
  every rule extracted by `adr-fmt --context`

## Consequences

- Easier: a rule carries its intervention layer through extraction.
- Harder: authors choose both ADR tier and rule layer; inherited classification
  cannot replace either judgment.
- Risks: valid numeric tags do not establish semantic correctness; higher-leverage
  tension is advisory, while equal/lower leverage passes T019.

Source evidence: `src/parser.rs:1041–1143` extracts tags and continuations;
`src/rules/template.rs:795–899` checks missing rules, ranges, IDs and tension;
`src/context.rs:337–354` preserves all eligible layers. Legacy bullet rules
are not an alternate tagged format. These checks do not prove annotation quality.
