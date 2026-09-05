# AFM-0033. Preamble Date Format Contract

Date: 2026-09-05
Last-reviewed: 2026-09-05
Tier: B
Status: Accepted

## Related

References: AFM-0003, AFM-0017, AFM-0032

## Context

`Date:` and `Last-reviewed:` are checked for presence by T002 and T003,
whose catalog descriptions have always read `(YYYY-MM-DD)`. Nothing
enforced that parenthesis. `Date: banana`, `Last-reviewed: never` and
`Date: 2026-13-45` each produced zero diagnostics, so the format the
guidance advertised and the format the tool accepted were different
formats, and no ADR said which one governed.

The defect class worth catching is a transposed digit. `2062-04-25` and
`0226-04-25` are shape-valid, sort wrongly, and read as plausible until
someone computes a review interval from them. A calendar check alone
does not catch either, so the contract needs a year bound as well.

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
  `date()` and `last_reviewed()` MUST keep returning the file's own
  text byte for byte at every verdict, and the verdict MUST be exposed
  only through crate-private accessors

R5 [5]: Date validation MUST be implemented with the standard library
  only. Month lengths, Gregorian leap years and a bounded year range
  are arithmetic, and a date crate would widen the dependency surface
  for no capability this contract needs

## Consequences

The advertised format becomes the enforced format, and the T002/T003
descriptions stop over-claiming. Every ADR in the corpus already
satisfies R1, so ratification changes no existing count and the rule
starts life with no backlog to clear.

R4 keeps this change invisible to `adr-srv`. The accessors AFM-0032:R5
pins are unchanged in signature and in meaning, so the validity verdict
is a crate-internal fact that no consumer can observe or come to depend
on. R5's cost is that leap-year arithmetic is written here rather than
depended upon; the bounded range makes that arithmetic small enough
that the trade favours the narrower dependency surface.
