# AFM-0015. Foundation Domain Inclusion in Context Resolution

Date: 2026-04-28
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0008

## Context

The `--context` mode resolves which ADR rules apply to a given
crate. AFM-0008:R2 establishes the configured domains whose ADRs form
the units of context selection; foundation inclusion specializes
that domain model. Domain configuration in `adr-fmt.toml` maps
crate names to domains, and each domain's ADRs contribute rules.
Domains may be marked `foundation = true`, meaning their rules
apply universally regardless of direct domain membership. This
corpus configures only AFM and declares no foundation domains.
Without foundation domain inclusion, cross-cutting concerns like
commit conventions and Rust idioms would need explicit mapping to
every crate, violating DRY and risking incomplete coverage when
new crates are added.

## Decision

Foundation domains are always included in `--context` output
alongside the crate's directly mapped domain.

R1 [5]: `--context` resolution collects rules from the crate's
  mapped domain plus all domains where `foundation = true` in
  `adr-fmt.toml`
R2 [5]: Only ADRs with status Accepted contribute rules to
  `--context` output; Draft, Proposed, Deprecated, and Superseded
  ADRs are excluded
R3 [5]: Within each included, non-stale Accepted ADR, emit every
  decision rule regardless of its tagged layer or the ADR's declared
  tier; tier tension is advisory lint, not an extraction filter
R4 [6]: Foundation domain inclusion is configured declaratively
  in `adr-fmt.toml`, not hard-coded; adding or removing foundation
  status requires only a configuration change

## Consequences

- Easier: newly mapped crates inherit configured foundation rules without
  repeating every cross-cutting mapping. An empty foundation set adds nothing.
- Harder: foundation rules cannot be selectively excluded per crate.
- Risks: an overbroad foundation declaration propagates obligations beyond
  their intended consumers; configuration requires semantic review.

Source evidence: `src/context.rs:115–169,337–354` selects Accepted non-stale
records and emits every eligible rule, retaining the all-layer contract.
`adr-fmt.toml:23–33` declares no foundation domain. This source review
establishes selection mechanics, not appropriateness of external mappings.
