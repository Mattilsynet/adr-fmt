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
//! Crate-private by construction (AFM-0026:R2). Nothing here is exported
//! at the crate root; doing so would be a new public item under
//! AFM-0026:R5 and would require an ADR.

/// One diagnostic's identity and its rendered governance description.
///
/// `summary` is the first rendered line and is always present;
/// `continuation` holds any further lines. Splitting them this way means a
/// description-less entry cannot be constructed.
pub(crate) struct RuleEntry {
    pub(crate) id: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) continuation: &'static [&'static str],
}

const fn entry(id: &'static str, summary: &'static str) -> RuleEntry {
    RuleEntry {
        id,
        summary,
        continuation: &[],
    }
}

const fn wrapped(
    id: &'static str,
    summary: &'static str,
    continuation: &'static [&'static str],
) -> RuleEntry {
    RuleEntry {
        id,
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
        "legacy ## Status section carrying a value MUST satisfy",
        "this; it fires only when neither supplies a status",
    ],
);
pub(crate) const T005C: RuleEntry = wrapped(
    "T005c",
    "Legacy ## Status section — accepted but deprecated. With",
    &[
        "no metadata field the section supplies the status and you",
        "SHOULD migrate it to a Status: preamble field; when a",
        "Status: field is also present the section is dead content",
        "and MUST be deleted — the metadata field is authoritative",
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
        "The effective minimum and maximum MUST be the",
        "configured T015 base scaled by the tier factor, so",
        "the minimum printed above and the minimum enforced",
        "here are one value. S-tier ADRs get more room,",
        "D-tier must be tighter. Flags the section for review.",
    ],
);
pub(crate) const T016: RuleEntry = wrapped(
    "T016",
    "Tagged rules — tier-scaled signal, not gate. Max rules",
    &[
        "scales with tier. Word count (7–60), sequential IDs",
        "and layer outside 1–12 (invalid Meadows leverage",
        "point) are warnings. A rule-shaped line that",
        "does not match the required `RN [L]: text` format",
        "is reported against its own line — it is not",
        "silently skipped. Exceeding may indicate the ADR",
        "covers multiple decisions.",
    ],
);
pub(crate) const T019: RuleEntry = wrapped(
    "T019",
    "Rule-tier tension — asymmetric leverage bound. T019 MUST",
    &[
        "fire if and only if the rule's layer-derived tier has",
        "higher leverage than the ADR's tier (rule_rank <",
        "adr_rank); equal or lower leverage MUST pass silently.",
        "T019 MUST NOT apply domain carve-outs and MUST NOT apply",
        "a distance threshold. Move the rule to a matching-",
        "tier ADR or adjust the layer annotation.",
    ],
);
pub(crate) const T020: RuleEntry = wrapped(
    "T020",
    "Reference load — tier-scaled limit on References:",
    &[
        "count. Root and Supersedes are structural and don't",
        "count. High reference count signals broad scope.",
    ],
);
pub(crate) const T022: RuleEntry = wrapped(
    "T022",
    "MADR residue section — headings such as `## Context and",
    &[
        "Problem Statement`, `## Decision Drivers`, `## Considered",
        "Options`, `## Decision Outcome` and `## Pros and Cons of",
        "the Options` are not part of this template. Fold their",
        "content into `## Context` or `## Decision` and remove the",
        "heading. Skipped on stale ADRs.",
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
        "unrecognized verb, or unparseable target. The malformed",
        "segment is skipped; other valid segments on the same",
        "line still parse and link. A clause-level target",
        "(`ID:Rn`) is accepted on `References:` only",
        "(AFM-0029:R4); elsewhere it is unparseable.",
    ],
);
pub(crate) const P004: RuleEntry = wrapped(
    "P004",
    "Duplicate ADR id: two records claim the same PREFIX-NNNN.",
    &[
        "Detected once, before any rule runs — no rule can consume",
        "the corpus while this holds (AFM-0008:R3).",
    ],
);

pub(crate) const N001: RuleEntry = entry(
    "N001",
    "Filename matches `PREFIX-NNNN-kebab-slug.md` pattern",
);
pub(crate) const N002: RuleEntry = wrapped(
    "N002",
    "Filename ID matches the H1 title ID — the filename and the",
    &["`# PREFIX-NNNN. Title` heading MUST name the same ADR"],
);
pub(crate) const N003: RuleEntry = wrapped(
    "N003",
    "Slug is lowercase kebab-case — letters, digits and hyphens",
    &[
        "only, with at least one letter segment, rejecting leading,",
        "trailing and consecutive hyphens (AFM-0008:R4)",
    ],
);
pub(crate) const N004: RuleEntry = wrapped(
    "N004",
    "Prefix matches a domain registered in `adr-fmt.toml` under",
    &["`[[domains]]`; any unregistered prefix warns (AFM-0008:R2)"],
);

pub(crate) const L001: RuleEntry = entry("L001", "Dangling link — target ADR file not found");
pub(crate) const L003: RuleEntry = entry("L003", "Supersedes-status consistency");
pub(crate) const L006: RuleEntry = wrapped(
    "L006",
    "Legacy relationship verb — migrate to its replacement verb",
    &["(see Legacy verbs above; per AFM-0009)"],
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
/// one is a one-line restatement. Both renderings are pinned here rather
/// than in the renderer so the divergence is visible in one place.
pub(crate) const T020_LINK_SUMMARY: RuleEntry = entry(
    "T020",
    "Reference load — tier-scaled max on References: count",
);

pub(crate) const S004: RuleEntry = entry("S004", "enforces presence of `## Retirement`");
pub(crate) const S005: RuleEntry = wrapped(
    "S005",
    "active ADR carries `## Retirement` — the section is for",
    &["stale ADRs only; delete it or retire the ADR"],
);
pub(crate) const S006: RuleEntry = wrapped(
    "S006",
    "terminal-status ADR is not in the stale directory — move",
    &["the file here and add a `## Retirement` section"],
);
pub(crate) const S007: RuleEntry = entry("S007", "enforces stub structure (sections + verbs)");
pub(crate) const S008: RuleEntry = wrapped(
    "S008",
    "stale ADR carries a non-terminal status — either set a",
    &[
        "terminal status (Rejected, Deprecated, Superseded by",
        "PREFIX-NNNN) or move the file back out of this directory",
    ],
);

pub(crate) const TEMPLATE_RULES: &[&RuleEntry] = &[
    &T002, &T003, &T004, &T005, &T005C, &T006, &T007, &T008, &T009, &T010, &T011, &T014, &T015,
];

pub(crate) const TEMPLATE_RULES_TAGGED: &[&RuleEntry] = &[&T016, &T019, &T020, &T022];

pub(crate) const PARSER_RULES: &[&RuleEntry] = &[&P001, &P002, &P003, &P004];

pub(crate) const NAMING_RULES: &[&RuleEntry] = &[&N001, &N002, &N003, &N004];

pub(crate) const LINK_RULES: &[&RuleEntry] = &[
    &L001,
    &L003,
    &L006,
    &L007,
    &L008,
    &L009,
    &L010,
    &L011,
    &L012,
    &L013,
    &L014,
    &L015,
    &L016,
    &L017,
    &L018,
    &L019,
    &L020,
    &T020_LINK_SUMMARY,
];

pub(crate) const STALE_RULES: &[&RuleEntry] = &[&S004, &S005, &S006, &S007, &S008];
