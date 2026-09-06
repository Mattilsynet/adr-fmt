# AFM-0024. Agent Critique via `--refs`

Date: 2026-05-03
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Supersedes: AFM-0021
References: AFM-0012, AFM-0011

## Context

AFM-0012:R4 defines the tension diagnostic this critique workflow must use,
so it is the constraining parent; AFM-0011:R3 supplies tier interpretation.
`--refs` answers an inbound graph question while agents read selected bodies
directly. Separating retrieval from judgment avoids presenting a graph
projection as an architectural critique.

## Decision

Agents can perform ADR critique by composing `--refs` calls with
targeted file reads; the binary does not inline bodies or compute
tier-tension summaries inline.

R1 [5]: An agent critiquing ADR-X issues `adr-fmt --refs ADR-X` to
  obtain the reverse-reference list, then reads individual referrer
  files selectively based on tier rank, status, and relevance — the
  binary's role is the graph projection, not content aggregation
R2 [5]: The asymmetric T019 rule (AFM-0012:R4) is the authoritative
  tier-tension diagnostic; agents surface T019 findings via
  `adr-fmt --lint` rather than by computing tension inline in the
  critique loop
R3 [6]: `adr-fmt --refs ADR-X` returns a one-bullet-per-referrer
   markdown list of non-stale References/Supersedes referrers without body content;
  agents that need body content follow up with direct file reads,
  preserving context-window budget

## Consequences

- Easier: agents enumerate inbound citations before selectively reading bodies.
- Harder: semantic critique still requires source reads and relevance judgment.
- Risks: a deterministic list does not guarantee sufficient context or sound
  conclusions; no context-window saving is measured here.

Source evidence: `src/refs.rs:32–101` selects and sorts non-stale referrers;
`src/lib.rs:292–317` rejects incomplete retrieval before rendering.
AFM-0012:R4 remains the live tension authority, not retired AFM-0021 rules.
Tier B classifies this information-flow workflow under AFM-0011:R1; no new
extensibility or semantic-analysis contract is implied.
