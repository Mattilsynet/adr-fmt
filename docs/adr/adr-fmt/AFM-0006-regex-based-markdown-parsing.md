# AFM-0006. Regex-Based Markdown Parsing Over AST Parsing

Date: 2026-04-27
Last-reviewed: 2026-09-06
Tier: D
Status: Accepted

## Related

References: AFM-0004

## Context

AFM-0004:R1 defines the structure this parser must extract, making it the
constraining parent. ADRs use ATX headings, fences and `Key: Value` metadata;
line-oriented extraction is sufficient for that limited contract. Fixed-shape
token validation is separate from markdown structure, and full Markdown
interpretation is not promised.

## Decision

Parse ADR markdown using line-by-line iteration with compiled regex
patterns. Do not depend on any markdown AST parser.

R1 [9]: Compiled regex patterns drive markdown structural
  extraction (headings, sections, fences, relationships) under an
  enum state machine; lexical validators for fixed-shape tokens
  (ADR IDs, layer numbers) may use byte-level checks
R2 [9]: Word counting accumulates per-section, excluding lines
  inside fenced code blocks by tracking fence open/close state
R3 [9]: Only ATX headings and fenced code blocks are supported;
  setext headings and indented code blocks are not recognized
R4 [12]: Reassess if ADRs require tables or structure complex
  enough that line-oriented parsing produces ambiguous results

## Consequences

- Easier: line-oriented extraction keeps the supported subset inspectable.
- Harder: authors cannot rely on full Markdown structure recognition.
- Risks: new syntax can create ambiguity; reassess under AFM-0006:R4.

Source evidence: `src/parser.rs:427–542,1041–1143` shows extraction and
Decision termination at the next H2. These source pointers do not establish
full Markdown equivalence, a performance bound or semantic validity.
