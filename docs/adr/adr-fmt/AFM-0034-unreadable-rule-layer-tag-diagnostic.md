# AFM-0034. Unreadable Rule Layer Tag Diagnostic

Date: 2026-09-05
Last-reviewed: 2026-09-05
Tier: B
Status: Accepted

## Related

References: AFM-0017, AFM-0003, AFM-0012
## Context

The tagged-rule regex AFM-0012:R2 pins requires a digit run as the
layer tag. A Decision line such as `R2 [abc]: text` does not match and
never reaches the rule id sequence. That sequence runs `R1`, `R3`, and
T016 reports a gap after `R1`.

That points at the wrong thing. The author mistyped one layer tag; the
tool answers that a rule is missing. A confident report of a
different fault beats no report only in volume.

Widening the regex was refuted. A layer type bounded to `1..=12`
cannot represent what the pinned regex admits — a planted
`R99999999999999999999` parses today — so the bound would reject input
the parser accepts, changing AFM-0012:R2 rather than serving it.
## Decision

Report the unreadable layer tag as a parser-stage diagnostic emitted
alongside the pinned regex, leaving the regex itself untouched.

R1 [5]: Emit `P005` at warning severity when a Decision line matches
  `RN [L]: text` but its `L` is not the run of digits AFM-0012:R2
  requires. `P005` belongs to the parser-stage namespace per
  AFM-0017:R1 because the tag failed to parse

R2 [5]: The `P005` message MUST name the rule id and quote the layer
  tag verbatim, and MUST state that the line is not read as a tagged
  rule, so the reader can connect it to any apparent gap in the
  surrounding sequence

R3 [5]: `P005` MUST NOT change which lines count as tagged rules. The
  regex AFM-0012:R2 pins stays verbatim, the line stays malformed, and
  the diagnostic is added beside that outcome rather than replacing it

R4 [5]: `P005` MUST NOT suppress the T016 sequence gap. The gap is a
  true statement about the rules that parsed; `P005` supplies the
  reason it exists, and suppression would need the parser to feed
  unparsed tags into a rule-level check

## Consequences

A mistyped layer tag now produces a diagnostic that names the mistyped
tag, on the line carrying it. The T016 gap remains and is no longer
the only signal, so the pair reads as one story rather than as a
misdirection.

The cost is that one authoring error yields three diagnostics: `P005`,
the T016 format complaint, and the T016 gap. That is accepted over
narrowing T016, which would be a semantic change to a rule for the
sake of output volume. R3 keeps the blast radius at one added
diagnostic: no existing rule changes meaning, and no ADR in the corpus
changes count.
