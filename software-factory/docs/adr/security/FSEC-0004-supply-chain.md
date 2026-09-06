# FSEC-0004. Supply Chain

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FSEC-0004

## Context

Dependencies introduce executable authority, maintenance exposure and redistribution obligations beyond their immediate API value.

## Decision

Dependency acceptance is a documented risk decision with continuing review obligations.

R1 [5]: Intake MUST record provenance, license compatibility, required features and transitive exposure before accepting a dependency.
R2 [5]: Build-time executable code MUST receive source review before execution when added or changed by dependency intake.
R3 [5]: Advisory and license checks MUST run locally and in delivery gates; exceptions MUST identify an owner, rationale, expiry and compensating controls.

## Consequences

Automated checks support but do not replace source review, legal judgment or maintenance assessment.
