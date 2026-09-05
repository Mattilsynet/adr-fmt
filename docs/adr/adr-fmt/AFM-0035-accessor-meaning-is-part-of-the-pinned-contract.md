# AFM-0035. Accessor Meaning Is Part Of The Pinned Contract

Date: 2026-09-05
Last-reviewed: 2026-09-05
Tier: S
Status: Accepted

## Related

References: AFM-0032, AFM-0026, AFM-0027

## Context

AFM-0032:R5 pins the trait surfaces, accessor signatures and
constructor signatures introduced by that ADR: none may be removed or
reshaped without a successor ADR. It pins shape. Nothing pins what a
call returns.

A change can therefore leave every signature intact and still break
`adr-srv`. `date()` keeps returning `Option<&str>` while beginning to
return `None` for a value the file carries, or a normalised form
rather than the file's own text. The crate compiles, this repository's
tests pass, the pinned surface is untouched by AFM-0032:R5's wording,
and the consumer AFM-0027 governs silently changes behaviour.

The gap has been closed at review time before, by a reviewer noticing
and asking. That works while someone notices. It is a judgement call
standing in for a rule, re-made more than once — the signal that it
should be written down.

The date contract ratified in AFM-0033 is a worked example: it
computes a validity verdict for `Date:` and deliberately routes it to
a crate-private accessor so the pinned public ones keep returning the
file's text byte for byte.

## Decision

Extend AFM-0032:R5 so that the observable meaning of a pinned item is
pinned alongside its signature.

R1 [5]: A change to what a pinned accessor returns for an input it
  already accepts IS a breaking change, whether or not the signature
  moves, and MUST be treated exactly as AFM-0032:R5 treats a reshape —
  it requires a successor ADR

R2 [5]: R1 covers the value, its emptiness, and its provenance. A
  pinned accessor that returned the source text MUST NOT begin
  returning a normalised, defaulted or derived form, and one that
  returned `Some` for an input MUST NOT begin returning `None` for it

R3 [5]: New internal knowledge about a pinned field MUST reach callers
  through a new accessor rather than by re-interpreting an existing
  one. Where that knowledge is crate-internal it MUST use a
  crate-private accessor, per the AFM-0033:R4 precedent

R4 [5]: A change claiming to preserve meaning MUST carry a test that
  pins the accessor's output across the inputs whose interpretation
  moved, so the claim is discharged by the suite rather than by review

## Consequences

The reviewer question becomes a rule with a named remedy: add an
accessor rather than repurpose one. R3 makes the cheap path also the
correct path, so the pressure that produces silent semantic drift is
removed rather than merely watched for.

The cost is accessor count. Encapsulation that computes a verdict must
expose it beside the raw value instead of folding it in, which is more
surface than the folded version. AFM-0026:R2 keeps that surface
crate-private unless an external consumer needs it, so the growth is
mostly internal and does not widen the v0.1 contract.

R4 is the part that bites. "Behaviour is unchanged" is not reviewable
by reading a diff once the change is more than a rename, so the claim
has to be executable. This does not reopen AFM-0026:R3, which defers
accessor migration to v0.2 and requires its own successor ADR; it
governs how meaning may move in the meantime.
