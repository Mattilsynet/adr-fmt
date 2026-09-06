# FSEC-0004. Supply Chain

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0004

## Context

Dependencies introduce executable authority, maintenance exposure and redistribution obligations beyond their immediate API value.

This root governs dependency acceptance and continuing checks, independently of application trust mechanisms. It supplies no approved dependency list and needs no deployment-specific parent.

## Decision

Dependency acceptance is a documented risk decision with continuing review obligations.

R1 [5]: Dependency intake MUST record provenance, license compatibility, required features and transitive exposure before accepting a dependency.
R2 [5]: Build-time executable code MUST receive source review before execution when added or changed by dependency intake.
R3 [5]: Advisory and license checks MUST run locally and in delivery gates; exceptions MUST identify an owner, rationale, expiry and compensating controls.

## Consequences

+ becomes easier: dependency acceptance and exceptions have identifiable review obligations.
− becomes harder: pre-execution source review and recurring checks add intake and maintenance work.
risks/migration: clean advisories do not establish source safety, license compatibility or future maintenance.

Evidence: [SOURCES.md](../../../SOURCES.md), Primary external sources, distinguishes reviewed documentation versions from approved dependencies. Review intake records, build-code reads, local/delivery checks and exception expiry against R1–R3; no dependency is accepted or legally cleared by this corpus migration.
