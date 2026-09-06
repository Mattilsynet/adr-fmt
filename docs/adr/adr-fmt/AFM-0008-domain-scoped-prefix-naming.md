# AFM-0008. Domain-Scoped Prefix Naming Convention

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0001

## Context

AFM-0001:R2 assigns domain registration to configuration; naming specializes
that registry, so it is the first-parent constraint. A configured prefix
identifies a domain and a four-digit number identifies a decision within it.
Kebab slugs add readable context. Numeric sorting is not proof of creation
chronology; authors still own non-recycling allocation and domain boundaries.

## Decision

Every ADR filename follows `PREFIX-NNNN-kebab-slug.md` where PREFIX
is a configured domain code and NNNN is a zero-padded sequence
number.

R1 [5]: Match filename to `PREFIX-NNNN-kebab-slug.md` and confirm
  H1 title contains the same `PREFIX-NNNN` identifier —
  validated by N001, N002, N003
R2 [5]: Record domain prefixes in `adr-fmt.toml` under `[[domains]]`
  and permit `adr-fmt` to trigger warning N004 for any unregistered prefix
R3 [5]: Bind permanent, non-recycling sequence numbers within each
  domain and permit gaps left by rejected or superseded ADRs
R4 [5]: Name slug segments as lowercase kebab-case — letters, digits,
  and hyphens only, with at least one letter segment, rejecting
  leading, trailing, and consecutive hyphens (validated by N003)
R5 [5]: Scope link integrity to targets carrying a configured
  `[[domains]]` prefix; a citation naming any other prefix addresses
  another corpus's governance, is unresolvable here by construction,
  and does not trigger L001

## Consequences

- Easier: configured prefixes scope resolution; new domains need configuration,
  not a hard-coded naming branch.
- Harder: authors coordinate permanent numbers and maintain registrations.
- Risks: foreign-prefix silence is not proof of target existence or entailment;
  four-digit capacity is finite, not a universal adequacy claim.

Source evidence: `src/rules/naming.rs:17–97` checks filenames and titles;
`src/rules/links.rs:93–143` distinguishes governed absence from foreign links;
`adr-fmt.toml:23–33` registers AFM. These checks do not reconstruct historical
number allocation or independently verify foreign governance.
