# AFM-0034. Unreadable Rule Layer Tag Diagnostic

Date: 2026-09-05
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0017, AFM-0003, AFM-0012
## Context

AFM-0017:R1 is the constraining parent because an unreadable tag belongs in
the parser diagnostic namespace. AFM-0012:R2 requires a digit run; `R2 [abc]`
therefore cannot become a parsed rule. In an R1/R3 sequence the missing R2
still causes T016. P005 identifies the malformed source rather than suppressing
that independent finding. Numeric overflow remains distinct: AFM-0039:R4 retains
its source spelling without narrowing the pinned parser language.
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

+ becomes easier: malformed tags have source-local explanations.
− becomes harder: an unreadable middle rule can produce P005 plus T016 format
  and sequence findings; the count depends on surrounding parsed rules.
risks/migration: P005 does not diagnose numeric overflow or establish rule meaning.

Evidence: `src/parser.rs:977–1143` classifies and retains malformed candidates;
`src/rules/template.rs:773–874` checks format, layer and parsed sequence.
These paths preserve diagnostic distinctions, not a universal three-warning count.
