# FGND-0003. Policy Lifecycle

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

Root: FGND-0003

## Context

Accumulated directives become conflicting authority unless ownership, applicability and retirement remain visible to their consumers.

This root governs policy maintenance itself, separately from work execution and feedback. Its applicability does not depend on a fabricated parent relationship.

## Decision

Policies are maintained contracts with explicit applicability, evidence surfaces and a responsible lifecycle owner.

R1 [3]: Each adopted policy MUST identify its scope and accountable owner; optional profiles MUST require explicit adoption.
R2 [3]: Binding obligations MUST name a checkable artifact or review criterion rather than relying on aspiration alone.
R3 [3]: Owners MUST review policies when contrary evidence appears and update current-state prose or retire the obsolete contract.

## Consequences

+ becomes easier: consumers can identify policy scope, owners and obsolete authority.
− becomes harder: adoption creates continuing review and retirement work.
risks/migration: Accepted distribution status does not appoint an adopter's owner or demonstrate that obsolete policies were removed.

Evidence: [README.md](../../../README.md), Adopt into a fresh repository, describes explicit adoption; [adr-fmt.toml](../../../adr-fmt.toml) separates core foundations from optional profiles. Review ownership and disposition records against R1–R3. Configuration and lint establish neither ownership nor retirement truth.
