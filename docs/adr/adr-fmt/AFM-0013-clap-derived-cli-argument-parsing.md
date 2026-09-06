# AFM-0013. Clap-Derived CLI Argument Parsing

Date: 2026-04-28
Last-reviewed: 2026-09-06
Tier: D
Status: Accepted

## Related

Supersedes: AFM-0002
References: AFM-0001

## Context

AFM-0001:R1 establishes binary authority; this decision specializes ownership
of its CLI definitions. Default guidance and lint, refs, context and tree
modes share help, version and exclusivity handling. Clap derive keeps those
declarations together without a parallel manual argument parser.

## Decision

Parse CLI arguments using clap's derive API. A single `Cli` struct
owns argument definitions and converts parsed input into `Mode`.

R1 [9]: Build all CLI argument definitions into a single `Cli`
  struct via clap's derive API, eliminating manual `args()`
  iteration outside that struct — exactly one struct owns all flags
R2 [9]: Declare mutually exclusive mode flags so clap enforces
  exclusivity at parse time rather than in runtime match logic
R3 [9]: Map each mode flag to the internal `Mode` enum via a
  conversion step after clap parsing — at least one conversion per
  flag variant
R4 [12]: Schedule a reassessment of `adr-fmt`'s `Cli` struct when
  the argument surface requires subcommands or dynamic argument
  generation beyond clap derive capabilities — file a new AFM ADR
  when that threshold is reached

## Consequences

- Easier: derive generates help, version and exclusivity handling.
- Harder: CLI maintenance depends on clap and its derive conventions.
- Risks: argument acceptance alone does not prove correct dispatch.

Source evidence: `src/lib.rs:62–114,594–643` defines `Cli`, converts modes
and tests mappings. `--max-warning-docs` requires lint; no `--depth` or
`--guidelines` flag exists. No compile-time, binary-size or usability
measurement is claimed.
