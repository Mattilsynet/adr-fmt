# AFM-0017. Parser-Stage Diagnostic Namespace

Date: 2026-04-29
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0003, AFM-0006, GND-0005

## Context

AFM-0003:R2 supplies the advisory stream and is the constraining parent:
per-file parsing problems must remain visible without aborting lint on the
first malformed document. AFM-0006:R1 supplies extraction mechanics. A distinct
namespace identifies parser findings before ordinary rule evaluation;
GND-0005 remains a foreign citation, not independently inspected evidence.

## Decision

Surface parser-stage failures as advisory diagnostics in a dedicated
`P###` rule-code namespace, merged with rule diagnostics in the
unified `--lint` output stream.

R1 [5]: Define parser-stage diagnostic codes in the `P###`
  namespace, distinct from `T###` (template), `L###` (links),
   and `I###` (integrity), to signal parser-stage failures or
   malformed input rather than ordinary rule-check findings
R2 [5]: Emit `P001` via `Diagnostic::warning("P001", path, 0, msg)`
  in `src/parser.rs` when a file matches the
  domain prefix filename pattern but `fs::read_to_string` fails;
  the ADR is excluded from rule checks for that run
R3 [5]: Emit `P002` via `Diagnostic::warning("P002", path, 0, msg)`
  in `src/parser.rs` when a file is readable but
  contains no `# PREFIX-NNNN. Title` H1 header recognized by
  `parse_title`; the ADR is excluded from rule checks for that run
R4 [5]: Return `Result<ParseOutcome, ParseError>` from `parse_domain`
   and `parse_stale`; reserve `Err` for failure to open the directory,
   routed through AFM-0003:R1 infrastructure handling; report per-entry
   read failures as P001 diagnostics in the returned outcome
R5 [6]: Merge parser diagnostics with rule diagnostics in
   `src/lib.rs` lint dispatch before calling
   `output::render_diagnostics` so the AFM-0003:R3 stdout
  contract surfaces a single combined `## Diagnostics` block

## Consequences

- Easier: lint reports parser findings alongside rule findings.
- Harder: consumers distinguish successfully parsed records from incomplete
  input; retrieval refuses parser-diagnostic corpora under AFM-0039:R2.
- Risks: an entry error may identify only its directory, not a filename;
  warning detail limits do not make missing evidence clean.

Source evidence: `src/parser.rs:105–189,226–259,289–330` separates directory,
entry and file failures; `src/lib.rs:282–350` guards retrieval and merges lint.
P001–P099 remain reserved for parser-stage diagnostics. This review does not
reproduce every filesystem race or assert semantic validity of parsed records.
