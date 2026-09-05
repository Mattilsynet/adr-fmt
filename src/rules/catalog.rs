//! Typed rule catalog — the single place a diagnostic's id and its
//! human-readable description are stated.
//!
//! AFM-0004:R2 [L5] requires governance guidance be generated from the
//! same code structures that perform validation. Before this catalog the
//! descriptions lived as string literals in `guidelines.rs` while the ids
//! lived as string literals at the `Diagnostic` construction sites, so the
//! two sides could drift apart silently — and did, three times on the
//! record.
//!
//! Both sides now read these entries: validators take their rule id from
//! `RuleEntry::id`, and `guidelines.rs` renders the registry sections from
//! the section slices below. Deleting an entry breaks the validator that
//! emits it, which makes the coupling a compile error rather than a
//! convention.
//!
//! Severity lives here too. It used to be a bare `Diagnostic::warning`
//! choice made independently at each construction site, with no registry to
//! read, which is why the T016 guidance could claim a rule was an error
//! while the validator emitted a warning (adr-fmt-5qd). `RuleEntry::diagnostic`
//! is now the only place in the crate that decides a diagnostic's severity,
//! and the renderer states severity by formatting the same field.
//!
//! Crate-private by construction (AFM-0026:R2). Nothing here is exported
//! at the crate root; doing so would be a new public item under
//! AFM-0026:R5 and would require an ADR.

use std::path::Path;

use crate::report::{Diagnostic, Severity};

/// One rendered line of a rule's description.
///
/// `Severity` exists so a line that talks about how loudly a rule fires
/// takes the word from the entry's `severity` field instead of restating
/// it. Guidance therefore cannot claim a severity the validators do not
/// emit — the drift class recorded three times in adr-fmt-5qd.
pub(crate) enum RuleLine {
    Text(&'static str),
    Severity {
        before: &'static str,
        after: &'static str,
    },
}

/// How one entry appears in a rendered registry section.
///
/// A diagnostic occasionally needs a second wording in a second section.
/// `Alternate` supplies that wording while BORROWING the canonical entry for
/// identity and severity, so one diagnostic keeps one identity and one
/// severity decision. Modelling the second wording as another `RuleEntry`
/// would give a presentation-only copy the ability to construct diagnostics
/// and to drift from the canonical entry if `Severity` grows.
pub(crate) enum RuleRendering {
    Canonical(&'static RuleEntry),
    Alternate {
        entry: &'static RuleEntry,
        summary: &'static str,
        continuation: &'static [RuleLine],
    },
}

impl RuleRendering {
    pub(crate) fn entry(&self) -> &'static RuleEntry {
        match self {
            Self::Canonical(entry) | Self::Alternate { entry, .. } => entry,
        }
    }

    pub(crate) fn summary(&self) -> &'static str {
        match self {
            Self::Canonical(entry) => entry.summary,
            Self::Alternate { summary, .. } => summary,
        }
    }

    pub(crate) fn continuation(&self) -> &'static [RuleLine] {
        match self {
            Self::Canonical(entry) => entry.continuation,
            Self::Alternate { continuation, .. } => continuation,
        }
    }
}

/// One diagnostic's identity, severity, and rendered governance description.
///
/// `summary` is the first rendered line and is always present;
/// `continuation` holds any further lines. Splitting them this way means a
/// description-less entry cannot be constructed.
pub(crate) struct RuleEntry {
    pub(crate) id: &'static str,
    pub(crate) severity: Severity,
    pub(crate) summary: &'static str,
    pub(crate) continuation: &'static [RuleLine],
}

impl RuleEntry {
    /// Build this rule's diagnostic.
    ///
    /// The only site in the crate that turns a severity into a
    /// `Diagnostic`, so a rule's severity is decided by its catalog entry
    /// rather than by whichever constructor a call site reached for.
    pub(crate) fn diagnostic(
        &'static self,
        file: &Path,
        line: usize,
        message: String,
    ) -> Diagnostic {
        match self.severity {
            Severity::Warning => Diagnostic::warning(self.id, file, line, message),
        }
    }
}

const fn entry(id: &'static str, summary: &'static str) -> RuleEntry {
    RuleEntry {
        id,
        severity: Severity::Warning,
        summary,
        continuation: &[],
    }
}

const fn wrapped(
    id: &'static str,
    summary: &'static str,
    continuation: &'static [RuleLine],
) -> RuleEntry {
    RuleEntry {
        id,
        severity: Severity::Warning,
        summary,
        continuation,
    }
}

pub(crate) const T002: RuleEntry = entry("T002", "Date field present (YYYY-MM-DD)");
pub(crate) const T003: RuleEntry = entry("T003", "Last-reviewed field present (YYYY-MM-DD)");
pub(crate) const T004: RuleEntry = entry("T004", "Tier field present (S/A/B/C/D)");
pub(crate) const T005: RuleEntry = wrapped(
    "T005",
    "Status value present — a `Status:` metadata field or a",
    &[
        RuleLine::Text("legacy ## Status section carrying a value MUST satisfy"),
        RuleLine::Text("this; it fires only when neither supplies a status"),
    ],
);
pub(crate) const T005C: RuleEntry = wrapped(
    "T005c",
    "Legacy ## Status section — accepted but deprecated. With",
    &[
        RuleLine::Text("no metadata field the section supplies the status and you"),
        RuleLine::Text("SHOULD migrate it to a Status: preamble field; when a"),
        RuleLine::Text("Status: field is also present the section is dead content"),
        RuleLine::Text("and MUST be deleted — the metadata field is authoritative"),
    ],
);
pub(crate) const T006: RuleEntry =
    entry("T006", "Status is a recognized keyword (rejects Amended)");
pub(crate) const T007: RuleEntry = entry(
    "T007",
    "Related section present with at least one relationship",
);
pub(crate) const T008: RuleEntry = entry("T008", "Context section present");
pub(crate) const T009: RuleEntry = entry("T009", "Decision section present");
pub(crate) const T010: RuleEntry = entry("T010", "Consequences section present");
pub(crate) const T011: RuleEntry = entry("T011", "Code block size limit (max 20 lines)");
pub(crate) const T014: RuleEntry = entry(
    "T014",
    "Section order (Related → Context → Decision → Consequences)",
);
pub(crate) const T015: RuleEntry = wrapped(
    "T015",
    "Section word count — tier-scaled signal, not gate.",
    &[
        RuleLine::Text("The effective minimum and maximum MUST be the"),
        RuleLine::Text("configured T015 base scaled by the tier factor, so"),
        RuleLine::Text("the minimum printed above and the minimum enforced"),
        RuleLine::Text("here are one value. S-tier ADRs get more room,"),
        RuleLine::Text("D-tier must be tighter. Flags the section for review."),
    ],
);
pub(crate) const T016: RuleEntry = wrapped(
    "T016",
    "Tagged rules — tier-scaled signal, not gate. Max rules",
    &[
        RuleLine::Text("scales with tier. Word count (7–60), sequential IDs"),
        RuleLine::Text("and layer outside 1–12 (invalid Meadows leverage"),
        RuleLine::Severity {
            before: "point) are ",
            after: "s. A rule-shaped line that",
        },
        RuleLine::Text("does not match the required `RN [L]: text` format"),
        RuleLine::Text("is reported against its own line — it is not"),
        RuleLine::Text("silently skipped. Exceeding may indicate the ADR"),
        RuleLine::Text("covers multiple decisions."),
    ],
);
pub(crate) const T019: RuleEntry = wrapped(
    "T019",
    "Rule-tier tension — asymmetric leverage bound. T019 MUST",
    &[
        RuleLine::Text("fire if and only if the rule's layer-derived tier has"),
        RuleLine::Text("higher leverage than the ADR's tier (rule_rank <"),
        RuleLine::Text("adr_rank); equal or lower leverage MUST pass silently."),
        RuleLine::Text("T019 MUST NOT apply domain carve-outs and MUST NOT apply"),
        RuleLine::Text("a distance threshold. Move the rule to a matching-"),
        RuleLine::Text("tier ADR or adjust the layer annotation."),
    ],
);
pub(crate) const T020: RuleEntry = wrapped(
    "T020",
    "Reference load — tier-scaled limit on References:",
    &[
        RuleLine::Text("count. Root and Supersedes are structural and don't"),
        RuleLine::Text("count. High reference count signals broad scope."),
    ],
);
pub(crate) const T022: RuleEntry = wrapped(
    "T022",
    "MADR residue section — headings such as `## Context and",
    &[
        RuleLine::Text("Problem Statement`, `## Decision Drivers`, `## Considered"),
        RuleLine::Text("Options`, `## Decision Outcome` and `## Pros and Cons of"),
        RuleLine::Text("the Options` are not part of this template. Fold their"),
        RuleLine::Text("content into `## Context` or `## Decision` and remove the"),
        RuleLine::Text("heading. Skipped on stale ADRs."),
    ],
);
pub(crate) const T023: RuleEntry = wrapped(
    "T023",
    "Date value malformed — a `Date:` or `Last-reviewed:` value",
    &[
        RuleLine::Text("that is present MUST be `YYYY-MM-DD`, name a real"),
        RuleLine::Text("calendar day, and fall in the year range 2000–2100."),
        RuleLine::Text("Absence is T002/T003; this fires only on a value that"),
        RuleLine::Text("is present and is not a date. The raw text is preserved"),
        RuleLine::Text("unchanged for readers (AFM-0033)."),
    ],
);
pub(crate) const P001: RuleEntry =
    entry("P001", "ADR file unreadable (filesystem error during read)");
pub(crate) const P002: RuleEntry = entry(
    "P002",
    "Missing or malformed H1 title (\"# PREFIX-NNNN. Title\")",
);
pub(crate) const P003: RuleEntry = wrapped(
    "P003",
    "Malformed `## Related` segment: missing `Verb: ` separator,",
    &[
        RuleLine::Text("unrecognized verb, or unparseable target. The malformed"),
        RuleLine::Text("segment is skipped; other valid segments on the same"),
        RuleLine::Text("line still parse and link. A clause-level target"),
        RuleLine::Text("(`ID:Rn`) is accepted on `References:` only"),
        RuleLine::Text("(AFM-0029:R4); elsewhere it is unparseable."),
    ],
);
pub(crate) const P004: RuleEntry = wrapped(
    "P004",
    "Duplicate ADR id: two records claim the same PREFIX-NNNN.",
    &[
        RuleLine::Text("Detected once, before any rule runs — no rule can consume"),
        RuleLine::Text("the corpus while this holds (AFM-0008:R3)."),
    ],
);

pub(crate) const N001: RuleEntry = entry(
    "N001",
    "Filename matches `PREFIX-NNNN-kebab-slug.md` pattern",
);
pub(crate) const N002: RuleEntry = wrapped(
    "N002",
    "Filename ID matches the H1 title ID — the filename and the",
    &[RuleLine::Text(
        "`# PREFIX-NNNN. Title` heading MUST name the same ADR",
    )],
);
pub(crate) const N003: RuleEntry = wrapped(
    "N003",
    "Slug is lowercase kebab-case — letters, digits and hyphens",
    &[
        RuleLine::Text("only, with at least one letter segment, rejecting leading,"),
        RuleLine::Text("trailing and consecutive hyphens (AFM-0008:R4)"),
    ],
);
pub(crate) const N004: RuleEntry = wrapped(
    "N004",
    "Prefix matches a domain registered in `adr-fmt.toml` under",
    &[RuleLine::Text(
        "`[[domains]]`; any unregistered prefix warns (AFM-0008:R2)",
    )],
);

pub(crate) const L001: RuleEntry = entry("L001", "Dangling link — target ADR file not found");
pub(crate) const L003: RuleEntry = entry("L003", "Supersedes-status consistency");
pub(crate) const L006: RuleEntry = wrapped(
    "L006",
    "Legacy relationship verb — migrate to its replacement verb",
    &[RuleLine::Text("(see Legacy verbs above; per AFM-0009)")],
);
pub(crate) const L007: RuleEntry = entry("L007", "Stale reference — link to stale archive ADR");
pub(crate) const L008: RuleEntry = entry("L008", "Root self-reference mismatch");
pub(crate) const L009: RuleEntry = entry("L009", "Root + References coexistence");
pub(crate) const L010: RuleEntry = entry("L010", "Missing parent — non-Root ADR has no References");
pub(crate) const L011: RuleEntry = entry(
    "L011",
    "Cross-domain parent — first References target is in another domain",
);
pub(crate) const L012: RuleEntry = entry(
    "L012",
    "Non-Accepted parent — first References target is Draft/Proposed (advisory)",
);
pub(crate) const L013: RuleEntry = entry("L013", "Parent-edge cycle — chain forms a loop");
pub(crate) const L014: RuleEntry = entry("L014", "Unreachable from root — chain ends at non-root");
pub(crate) const L015: RuleEntry = entry(
    "L015",
    "Root-first heuristic — first ref is Root while specialized siblings exist",
);
pub(crate) const L016: RuleEntry = entry(
    "L016",
    "Lower-tier parent — parent's tier is weaker leverage than child's",
);
pub(crate) const L017: RuleEntry = entry(
    "L017",
    "Superseded parent — first References target is Superseded by another ADR",
);
pub(crate) const L018: RuleEntry = entry(
    "L018",
    "Parent-cross-domain mismatch — declaration ID does not match first References",
);
pub(crate) const L019: RuleEntry = entry(
    "L019",
    "Parent-cross-domain target missing — declared ADR does not exist",
);
pub(crate) const L020: RuleEntry = entry(
    "L020",
    "Link integrity indeterminate — target exists but failed to parse",
);

/// T020 as the LINK RULES section states it.
///
/// The same diagnostic is described twice in governance output, in
/// different words: the template section stresses which verbs count, this
/// one is a one-line restatement. Both wordings are pinned here rather than
/// in the renderer so the divergence is visible in one place, and this one
/// borrows canonical `T020` rather than declaring a second entry for it.
/// Unifying the two wordings would change generated governance output and is
/// not this crate's decision to take.
pub(crate) const T020_LINK_SUMMARY: RuleRendering = RuleRendering::Alternate {
    entry: &T020,
    summary: "Reference load — tier-scaled max on References: count",
    continuation: &[],
};

pub(crate) const S004: RuleEntry = entry("S004", "enforces presence of `## Retirement`");
pub(crate) const S005: RuleEntry = wrapped(
    "S005",
    "active ADR carries `## Retirement` — the section is for",
    &[RuleLine::Text(
        "stale ADRs only; delete it or retire the ADR",
    )],
);
pub(crate) const S006: RuleEntry = wrapped(
    "S006",
    "terminal-status ADR is not in the stale directory — move",
    &[RuleLine::Text(
        "the file here and add a `## Retirement` section",
    )],
);
pub(crate) const S007: RuleEntry = entry("S007", "enforces stub structure (sections + verbs)");
pub(crate) const S008: RuleEntry = wrapped(
    "S008",
    "stale ADR carries a non-terminal status — either set a",
    &[
        RuleLine::Text("terminal status (Rejected, Deprecated, Superseded by"),
        RuleLine::Text("PREFIX-NNNN) or move the file back out of this directory"),
    ],
);

pub(crate) const TEMPLATE_RULES: &[RuleRendering] = &[
    RuleRendering::Canonical(&T002),
    RuleRendering::Canonical(&T003),
    RuleRendering::Canonical(&T004),
    RuleRendering::Canonical(&T005),
    RuleRendering::Canonical(&T005C),
    RuleRendering::Canonical(&T006),
    RuleRendering::Canonical(&T007),
    RuleRendering::Canonical(&T008),
    RuleRendering::Canonical(&T009),
    RuleRendering::Canonical(&T010),
    RuleRendering::Canonical(&T011),
    RuleRendering::Canonical(&T014),
    RuleRendering::Canonical(&T015),
];

pub(crate) const TEMPLATE_RULES_TAGGED: &[RuleRendering] = &[
    RuleRendering::Canonical(&T016),
    RuleRendering::Canonical(&T019),
    RuleRendering::Canonical(&T020),
    RuleRendering::Canonical(&T022),
    RuleRendering::Canonical(&T023),
];

pub(crate) const PARSER_RULES: &[RuleRendering] = &[
    RuleRendering::Canonical(&P001),
    RuleRendering::Canonical(&P002),
    RuleRendering::Canonical(&P003),
    RuleRendering::Canonical(&P004),
];

pub(crate) const NAMING_RULES: &[RuleRendering] = &[
    RuleRendering::Canonical(&N001),
    RuleRendering::Canonical(&N002),
    RuleRendering::Canonical(&N003),
    RuleRendering::Canonical(&N004),
];

pub(crate) const LINK_RULES: &[RuleRendering] = &[
    RuleRendering::Canonical(&L001),
    RuleRendering::Canonical(&L003),
    RuleRendering::Canonical(&L006),
    RuleRendering::Canonical(&L007),
    RuleRendering::Canonical(&L008),
    RuleRendering::Canonical(&L009),
    RuleRendering::Canonical(&L010),
    RuleRendering::Canonical(&L011),
    RuleRendering::Canonical(&L012),
    RuleRendering::Canonical(&L013),
    RuleRendering::Canonical(&L014),
    RuleRendering::Canonical(&L015),
    RuleRendering::Canonical(&L016),
    RuleRendering::Canonical(&L017),
    RuleRendering::Canonical(&L018),
    RuleRendering::Canonical(&L019),
    RuleRendering::Canonical(&L020),
    T020_LINK_SUMMARY,
];

pub(crate) const STALE_RULES: &[RuleRendering] = &[
    RuleRendering::Canonical(&S004),
    RuleRendering::Canonical(&S005),
    RuleRendering::Canonical(&S006),
    RuleRendering::Canonical(&S007),
    RuleRendering::Canonical(&S008),
];
