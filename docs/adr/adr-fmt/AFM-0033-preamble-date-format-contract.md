# AFM-0033. Preamble Date Format Contract

Date: 2026-09-05
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0003, AFM-0017, AFM-0032

## Context

Presence, calendar validity and accessor meaning are separate contracts.
AFM-0003:R2 is the constraining parent: invalid dates produce advisory warnings,
not parse failure. T002/T003 own absence; T023 owns malformed present values.
The inclusive 2000–2100 range catches some year mistakes, not every transposition:
`2062-04-25` remains valid. AFM-0032:R5 pins accessor shape while this decision
keeps validity knowledge crate-private.

## Decision

Ratify `YYYY-MM-DD` as the format contract for both preamble date
fields, enforced by a new advisory `T023`.

R1 [5]: A `Date:` or `Last-reviewed:` value that is present MUST be ten
  characters of the form `YYYY-MM-DD`, MUST name a day that exists in
  that month under the proleptic Gregorian calendar, and MUST carry a
  year in the inclusive range 2000–2100

R2 [5]: Violations of R1 MUST surface as `T023` at warning severity per
  AFM-0003:R2, quoting the offending raw value and naming which of the
  two fields carried it, so the diagnostic is actionable without
  reopening the file

R3 [5]: `T023` MUST NOT subsume T002 or T003. An absent field is a
  presence failure and stays theirs; `T023` fires only on a value that
  is present and is not a date, so the two conditions never both fire
  for one field

R4 [5]: The parsed verdict MUST NOT reach the pinned public accessors.
  `date()` and `last_reviewed()` MUST preserve parser-supplied field text
  byte for byte at every verdict, without undoing preamble trimming or
  empty-value handling; the verdict MUST be exposed only through
  crate-private accessors

R5 [5]: Date validation MUST be implemented with the standard library
  only. Month lengths, Gregorian leap years and a bounded year range
  are arithmetic, and a date crate would widen the dependency surface
  for no capability this contract needs

## Consequences

+ becomes easier: authors distinguish absent fields from invalid dates without
  losing the retained value.
− becomes harder: standard-library calendar arithmetic needs maintained tests.
risks/migration: an in-range date can still be historically wrong; diagnostics
  are observable even though raw accessors keep their meaning.

Evidence: `src/model.rs:1243–1301,2244–2311` separates raw values and verdicts;
`src/parser.rs:569–581` trims preamble values and ignores empty values;
`src/rules/template.rs:241–276` emits presence or validity diagnostics.
Constructor tests do not prove literal file-whitespace preservation or downstream
consumer compatibility.
