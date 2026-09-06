# AFM-0032. AdrId/AdrRecord Encapsulation Amendment to AFM-0026

Date: 2026-08-12
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0026, AFM-0028

## Context

AFM-0026:R1 is the constraining parent because it names the public model
surface. Private fields prevent direct construction and mutation outside
`model`; they do not constrain the defining module itself. Validated `AdrId`
entry points enforce a 2–4 uppercase-ASCII prefix and number 0–9999.
`AdrRecord` represents parser observations, including malformed input, rather
than certifying every cross-field relationship.

The crate-visible parser constructor trusts supplied fields; test-only builders
and mutable accessors deliberately admit sentinels. These are distinct boundaries,
not universal construction safety. AFM-0028:R1 supplies error traits;
AFM-0036:R4 governs compatibility. This narrow amendment leaves the remaining
AFM-0026 surface in force and makes no downstream migration-completion claim.

## Decision

Keep the two model types encapsulated at the module boundary, with validated
public ID creation and parser-owned record acquisition.

R1 [5]: `AdrId` and `AdrRecord` fields MUST be private to `model`, not
  `pub` or `pub(crate)`. Code outside `model` MUST use constructors and
  accessors rather than field access or struct literals. Defining-module
  code and test-only construction remain trusted, not excluded by privacy.

R2 [5]: `AdrId::try_new` and `TryFrom<&str>` MUST reject prefixes outside
  2–4 uppercase ASCII letters and numbers above 9999 with `AdrIdError`.
  Parsing helpers MUST delegate validation while retaining `Option` signatures;
  Clone retains the existing value. `AdrIdError` MUST remain re-exported
  as `adr_fmt::AdrIdError` under AFM-0026:R1.

R3 [5]: External callers MUST obtain `AdrRecord` through `parse_domain`,
  `parse_stale` or cloning an existing record, not an external builder.
  Production crate code MUST use `AdrRecord::from_parser_fields` for assembly;
  its crate visibility does not enforce parser-exclusive access or cross-field
  consistency. Every field MUST have a read accessor; new fields MUST add
  accessors in the same change.

R4 [5]: `AdrIdError` — the error type returned by R2's constructors —
  inherits the AFM-0028:R1 trait floor by construction
  (`core::fmt::Display`, `core::fmt::Debug`, `std::error::Error`) as a
  new type added to the AFM-0026:R1 surface under AFM-0026:R5. No
  separate ADR is needed to establish this per AFM-0028:R4.

R5 [5]: The trait surfaces, accessor signatures, and constructor
  signatures introduced by this ADR are part of the v0.1 semver
  contract per the extension of AFM-0026:R3. New accessors or trait
  impls MAY be added in minor versions; none introduced here MAY be
  removed or reshaped without a successor ADR.

## Consequences

+ becomes easier: external callers cannot mutate model fields and receive
  nameable ID validation errors.
− becomes harder: fields need accessors and compatibility review.
risks/migration: defining-module literals, crate assembly and test mutations are
  trusted routes, not a general invariant proof; downstream builds are unverified.

Evidence: `src/model.rs:159–284,335–685,703–875,1523–1559` covers fields,
constructors, Clone, parsing helpers and test routes. Neither type implements
Default, serde construction, FromStr or production mutable accessors. The parser
calls assembly at `src/parser.rs:514–542`. Tests at `src/model.rs:2083–2144`
exercise valid/rejected IDs; lines 2220–2240 demonstrate that record assembly
permits independently supplied title/line facts. No compile-fail proof is claimed.
