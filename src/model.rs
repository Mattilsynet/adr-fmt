use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// A domain directory (e.g., `docs/adr/cherry/` with prefix `CHE`).
#[derive(Debug, Clone)]
pub struct DomainDir {
    pub path: PathBuf,
    pub prefix: String,
    pub name: String,
}

/// A tagged rule extracted from the Decision section.
///
/// Format in ADR: `R1 [5]: Rule text here`
/// Global identifier: `CHE-0042:R1:L5`
#[derive(Debug, Clone)]
pub struct TaggedRule {
    pub id: String,
    pub text: String,
    /// Meadows leverage layer (1-12). 0 indicates unparsed/invalid.
    pub layer: u8,
    /// 1-indexed line number where this rule appears in the source file.
    pub line: usize,
}

/// Map a Meadows leverage layer (1-12) to the corresponding tier.
///
/// Mapping: S=1-3, A=4, B=5-6, C=7-8, D=9-12.
/// Returns `None` for layer 0 or >12 (invalid).
#[must_use]
pub fn layer_to_tier(layer: u8) -> Option<Tier> {
    match layer {
        1..=3 => Some(Tier::S),
        4 => Some(Tier::A),
        5..=6 => Some(Tier::B),
        7..=8 => Some(Tier::C),
        9..=12 => Some(Tier::D),
        _ => None,
    }
}

/// Composite ADR identifier: prefix + number (e.g., CHE-0042).
///
/// # Invariants (enforced by construction)
///
/// Every `AdrId` satisfies:
/// - `prefix.len() ∈ 2..=4`
/// - every byte of `prefix` is `b'A'..=b'Z'` (ASCII uppercase)
/// - `number ∈ 0..=9999` (encoded as exactly 4 digits via `Display`)
///
/// Fields are private; the only construction paths are
/// [`AdrId::try_new`] and `TryFrom<&str>`, both of which validate the
/// invariants above and reject violations with [`AdrIdError`]. An
/// invalid `AdrId` has no constructor and therefore cannot exist. See
/// AFM-0032.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdrId {
    prefix: String,
    number: u16,
}

/// Error returned by fallible `AdrId` construction ([`AdrId::try_new`],
/// `TryFrom<&str>`). Implements `Display` + `Debug` + `std::error::Error`
/// per the AFM-0028:R1 trait floor. See AFM-0032.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdrIdError {
    /// `prefix.len()` was not in `2..=4`.
    PrefixLength {
        /// The rejected prefix.
        prefix: String,
    },
    /// `prefix` contained a byte outside `b'A'..=b'Z'`.
    PrefixNotUppercaseAscii {
        /// The rejected prefix.
        prefix: String,
    },
    /// `number` was outside `0..=9999`.
    NumberOutOfRange {
        /// The rejected number.
        number: u32,
    },
    /// The input string was not in `PREFIX-NNNN` form.
    Malformed {
        /// The rejected input string.
        input: String,
    },
}

impl fmt::Display for AdrIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrefixLength { prefix } => {
                write!(f, "AdrId prefix {prefix:?} must be 2-4 characters")
            }
            Self::PrefixNotUppercaseAscii { prefix } => {
                write!(f, "AdrId prefix {prefix:?} must be uppercase ASCII")
            }
            Self::NumberOutOfRange { number } => {
                write!(f, "AdrId number {number} is out of range 0..=9999")
            }
            Self::Malformed { input } => {
                write!(f, "AdrId string {input:?} is not in PREFIX-NNNN form")
            }
        }
    }
}

impl std::error::Error for AdrIdError {}

impl AdrId {
    /// Validated `AdrId` constructor.
    ///
    /// # Errors
    /// Returns [`AdrIdError::PrefixLength`] when `prefix.len()` is not
    /// in `2..=4`, [`AdrIdError::PrefixNotUppercaseAscii`] when
    /// `prefix` contains a non-uppercase-ASCII byte, or
    /// [`AdrIdError::NumberOutOfRange`] when `number > 9999`.
    pub fn try_new(prefix: &str, number: u16) -> Result<Self, AdrIdError> {
        let prefix_len = prefix.len();
        if !(2..=4).contains(&prefix_len) {
            return Err(AdrIdError::PrefixLength {
                prefix: prefix.to_owned(),
            });
        }
        if !prefix.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(AdrIdError::PrefixNotUppercaseAscii {
                prefix: prefix.to_owned(),
            });
        }
        if number > 9999 {
            return Err(AdrIdError::NumberOutOfRange {
                number: u32::from(number),
            });
        }
        Ok(Self {
            prefix: prefix.to_owned(),
            number,
        })
    }

    /// The domain prefix (e.g., `"CHE"`).
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// The numeric identifier (e.g., `42` for `CHE-0042`).
    #[must_use]
    pub fn number(&self) -> u16 {
        self.number
    }
}

impl TryFrom<&str> for AdrId {
    type Error = AdrIdError;

    /// Parse a strict ADR ID like `CHE-0042`.
    ///
    /// Accepts exactly `^[A-Z]{2,4}-[0-9]{4}$` — uppercase ASCII prefix
    /// of 2–4 letters, dash, exactly 4 digits, nothing else. No
    /// whitespace trimming; callers must pass clean input.
    ///
    /// # Errors
    /// Returns [`AdrIdError::Malformed`] when the input is not in
    /// `PREFIX-NNNN` form (missing separator, wrong digit count,
    /// non-digit characters); otherwise delegates prefix/number
    /// validation to [`AdrId::try_new`].
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let (prefix, num_str) = s.split_once('-').ok_or_else(|| AdrIdError::Malformed {
            input: s.to_owned(),
        })?;
        if num_str.len() != 4 || !num_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AdrIdError::Malformed {
                input: s.to_owned(),
            });
        }
        let number: u16 = num_str.parse().map_err(|_| AdrIdError::Malformed {
            input: s.to_owned(),
        })?;
        Self::try_new(prefix, number)
    }
}

impl fmt::Display for AdrId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:04}", self.prefix, self.number)
    }
}

/// Parsed ADR record with all metadata and line numbers.
///
/// Bool fields represent independent parser-detected facts about the
/// record (presence of various sections/headers); collapsing into a
/// flags enum would lose the per-field rustdoc surface and obscure
/// individual access patterns in rules.
#[derive(Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "AdrRecord is pinned to the R1 crate-root re-export set (AFM-0026:R1); the bools are independent parser section-presence facts read individually in rules — collapsing would widen the pinned surface beyond R1 without a successor ADR (AFM-0026:R5) and forces an adr-srv re-scrape (AFM-0027:R5)"
)]
pub struct AdrRecord {
    id: AdrId,
    file_path: PathBuf,
    title: Option<String>,
    title_line: usize,
    date: Option<String>,
    last_reviewed: Option<String>,
    tier: TierField,
    status: Option<Status>,
    status_line: usize,
    status_raw: Option<String>,
    related: Related,
    has_context: bool,
    has_decision: bool,
    has_consequences: bool,
    has_retirement: bool,
    /// True when the ADR file lives in the stale archive directory.
    is_stale: bool,
    /// True when status was parsed from the legacy `## Status` section
    /// (not the `Status:` preamble metadata field).
    status_from_section: bool,
    max_code_block_lines: usize,
    /// 1-indexed line number of the opening fence of the largest code
    /// block. 0 if no code blocks exist.
    max_code_block_line: usize,
    /// Ordered list of H2 section names as they appear in the file.
    section_order: Vec<String>,
    /// Word count per H2 section (section name → count). Code blocks
    /// are excluded from the count.
    section_word_counts: HashMap<String, usize>,
    /// Crates associated with this ADR via `Crates:` metadata field.
    crates: Vec<String>,
    /// Tagged rules extracted from the Decision section
    /// (`RN [L]: text` pattern). Empty when no tagged rules found.
    decision_rules: Vec<TaggedRule>,
    /// Cross-domain parent exception declared in the preamble via
    /// `Parent-cross-domain: PREFIX-NNNN — reason`. When present and
    /// matching the first `References:` target, suppresses L011
    /// (cross-domain parent edge) for that relationship.
    parent_cross_domain: CrossDomainParent,
}

impl AdrRecord {
    /// True if this ADR declares itself as a tree root via `Root: OWN-ID`.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.relationships()
            .iter()
            .any(|r| r.verb == RelVerb::Root && r.target == self.id)
    }

    /// This record's `AdrId`.
    #[must_use]
    pub fn id(&self) -> &AdrId {
        &self.id
    }

    /// Path to the source ADR file.
    #[must_use]
    pub fn file_path(&self) -> &std::path::Path {
        &self.file_path
    }

    /// Parsed title, if the preamble had a `# Title` heading.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Raw `Date:` preamble value, if present.
    #[must_use]
    pub fn date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    /// Raw `Last-reviewed:` preamble value, if present.
    #[must_use]
    pub fn last_reviewed(&self) -> Option<&str> {
        self.last_reviewed.as_deref()
    }

    /// Parsed `Tier:` preamble value, if present and valid.
    ///
    /// Returns `None` both when the field is absent and when its value
    /// is unrecognised; callers that must tell those apart use the
    /// crate-internal `tier_field`.
    #[must_use]
    pub fn tier(&self) -> Option<Tier> {
        self.tier.value()
    }

    pub(crate) fn tier_field(&self) -> &TierField {
        &self.tier
    }

    /// Parsed lifecycle status, if present.
    #[must_use]
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    /// Relationships declared in the `## Related` section.
    #[must_use]
    pub fn relationships(&self) -> &[Relationship] {
        self.related.relationships()
    }

    /// 1-indexed line number of the parsed title heading.
    #[must_use]
    pub fn title_line(&self) -> usize {
        self.title_line
    }

    /// 1-indexed line number of the parsed status line, if any.
    #[must_use]
    pub fn status_line(&self) -> usize {
        self.status_line
    }

    /// Raw, unparsed status text as it appeared in the source.
    #[must_use]
    pub fn status_raw(&self) -> Option<&str> {
        self.status_raw.as_deref()
    }

    pub(crate) fn related(&self) -> &Related {
        &self.related
    }

    /// True when a `## Context` section was found.
    #[must_use]
    pub fn has_context(&self) -> bool {
        self.has_context
    }

    /// True when a `## Decision` section was found.
    #[must_use]
    pub fn has_decision(&self) -> bool {
        self.has_decision
    }

    /// True when a `## Consequences` section was found.
    #[must_use]
    pub fn has_consequences(&self) -> bool {
        self.has_consequences
    }

    /// True when a `## Retirement` section was found.
    #[must_use]
    pub fn has_retirement(&self) -> bool {
        self.has_retirement
    }

    /// True when the ADR file lives in the stale archive directory.
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.is_stale
    }

    /// True when status was parsed from the legacy `## Status` section
    /// (not the `Status:` preamble metadata field).
    #[must_use]
    pub fn status_from_section(&self) -> bool {
        self.status_from_section
    }

    /// Line count of the largest fenced code block. 0 if none.
    #[must_use]
    pub fn max_code_block_lines(&self) -> usize {
        self.max_code_block_lines
    }

    /// 1-indexed line number of the opening fence of the largest code
    /// block. 0 if no code blocks exist.
    #[must_use]
    pub fn max_code_block_line(&self) -> usize {
        self.max_code_block_line
    }

    /// Ordered list of H2 section names as they appear in the file.
    #[must_use]
    pub fn section_order(&self) -> &[String] {
        &self.section_order
    }

    /// Word count per H2 section (section name → count).
    #[must_use]
    pub fn section_word_counts(&self) -> &HashMap<String, usize> {
        &self.section_word_counts
    }

    /// Crates associated with this ADR via `Crates:` metadata field.
    #[must_use]
    pub fn crates(&self) -> &[String] {
        &self.crates
    }

    /// Tagged rules extracted from the Decision section.
    #[must_use]
    pub fn decision_rules(&self) -> &[TaggedRule] {
        &self.decision_rules
    }

    /// Cross-domain parent exception declared via `Parent-cross-domain:`.
    ///
    /// Returns `Some` only for a well-formed declaration; a malformed
    /// one yields `None` and is reported by L018 rather than granting
    /// AFM-0020:R3 suppression.
    #[must_use]
    pub fn parent_cross_domain(&self) -> Option<&AdrId> {
        self.parent_cross_domain.honoured_id()
    }

    pub(crate) fn parent_cross_domain_field(&self) -> &CrossDomainParent {
        &self.parent_cross_domain
    }

    /// Narrow, in-module constructor used exclusively by the parser
    /// (`parser::parse_adr_file`) once all fields have been derived
    /// from the source file. Not part of the public API; establishes
    /// no additional invariants beyond what the parser itself derives.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors every AdrRecord field 1:1; this is the sole parser-facing constructor per AFM-0032:R3, not a general-purpose builder that would warrant decomposition"
    )]
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "mirrors AdrRecord's own independent parser-detected section-presence bools (see the struct-level clippy::struct_excessive_bools expect above); same rationale applies to the constructor that assembles them"
    )]
    pub(crate) fn from_parser_fields(
        id: AdrId,
        file_path: PathBuf,
        title: Option<String>,
        title_line: usize,
        date: Option<String>,
        last_reviewed: Option<String>,
        tier: TierField,
        status: Option<Status>,
        status_line: usize,
        status_raw: Option<String>,
        related: Related,
        has_context: bool,
        has_decision: bool,
        has_consequences: bool,
        has_retirement: bool,
        is_stale: bool,
        status_from_section: bool,
        max_code_block_lines: usize,
        max_code_block_line: usize,
        section_order: Vec<String>,
        section_word_counts: HashMap<String, usize>,
        crates: Vec<String>,
        decision_rules: Vec<TaggedRule>,
        parent_cross_domain: CrossDomainParent,
    ) -> Self {
        Self {
            id,
            file_path,
            title,
            title_line,
            date,
            last_reviewed,
            tier,
            status,
            status_line,
            status_raw,
            related,
            has_context,
            has_decision,
            has_consequences,
            has_retirement,
            is_stale,
            status_from_section,
            max_code_block_lines,
            max_code_block_line,
            section_order,
            section_word_counts,
            crates,
            decision_rules,
            parent_cross_domain,
        }
    }
}

#[cfg(test)]
impl AdrId {
    /// Test-only unchecked constructor. Bypasses [`AdrId::try_new`]'s
    /// validation deliberately: test fixtures build both valid and
    /// invariant-violating sentinels to exercise rules that operate on
    /// already-parsed records.
    pub(crate) fn test_new(prefix: impl Into<String>, number: u16) -> Self {
        Self {
            prefix: prefix.into(),
            number,
        }
    }
}

#[cfg(test)]
impl AdrRecord {
    pub(crate) fn id_mut(&mut self) -> &mut AdrId {
        &mut self.id
    }

    pub(crate) fn file_path_mut(&mut self) -> &mut PathBuf {
        &mut self.file_path
    }

    pub(crate) fn title_mut(&mut self) -> &mut Option<String> {
        &mut self.title
    }

    pub(crate) fn title_line_mut(&mut self) -> &mut usize {
        &mut self.title_line
    }

    pub(crate) fn date_mut(&mut self) -> &mut Option<String> {
        &mut self.date
    }

    pub(crate) fn last_reviewed_mut(&mut self) -> &mut Option<String> {
        &mut self.last_reviewed
    }

    pub(crate) fn set_tier(&mut self, tier: Option<Tier>) {
        self.tier = tier.map_or(TierField::Absent, TierField::Valid);
    }

    pub(crate) fn set_tier_field(&mut self, tier: TierField) {
        self.tier = tier;
    }

    pub(crate) fn status_mut(&mut self) -> &mut Option<Status> {
        &mut self.status
    }

    pub(crate) fn status_line_mut(&mut self) -> &mut usize {
        &mut self.status_line
    }

    pub(crate) fn status_raw_mut(&mut self) -> &mut Option<String> {
        &mut self.status_raw
    }

    pub(crate) fn set_related(&mut self, related: Related) {
        self.related = related;
    }

    pub(crate) fn relationships_mut(&mut self) -> &mut Vec<Relationship> {
        if !matches!(self.related, Related::Parsed(_)) {
            self.related = Related::Parsed(Vec::new());
        }
        match &mut self.related {
            Related::Parsed(v) => v,
            Related::Absent | Related::Malformed { .. } => unreachable!(),
        }
    }

    pub(crate) fn has_context_mut(&mut self) -> &mut bool {
        &mut self.has_context
    }

    pub(crate) fn has_decision_mut(&mut self) -> &mut bool {
        &mut self.has_decision
    }

    pub(crate) fn has_consequences_mut(&mut self) -> &mut bool {
        &mut self.has_consequences
    }

    pub(crate) fn has_retirement_mut(&mut self) -> &mut bool {
        &mut self.has_retirement
    }

    pub(crate) fn is_stale_mut(&mut self) -> &mut bool {
        &mut self.is_stale
    }

    pub(crate) fn status_from_section_mut(&mut self) -> &mut bool {
        &mut self.status_from_section
    }

    pub(crate) fn max_code_block_lines_mut(&mut self) -> &mut usize {
        &mut self.max_code_block_lines
    }

    pub(crate) fn max_code_block_line_mut(&mut self) -> &mut usize {
        &mut self.max_code_block_line
    }

    pub(crate) fn section_order_mut(&mut self) -> &mut Vec<String> {
        &mut self.section_order
    }

    pub(crate) fn section_word_counts_mut(&mut self) -> &mut HashMap<String, usize> {
        &mut self.section_word_counts
    }

    pub(crate) fn crates_mut(&mut self) -> &mut Vec<String> {
        &mut self.crates
    }

    pub(crate) fn decision_rules_mut(&mut self) -> &mut Vec<TaggedRule> {
        &mut self.decision_rules
    }

    pub(crate) fn set_parent_cross_domain(&mut self, declared: CrossDomainParent) {
        self.parent_cross_domain = declared;
    }
}

#[cfg(test)]
impl AdrRecord {
    /// Test-only sentinel builder. Not `Default` — an `AdrId` with an
    /// empty `prefix` violates the invariants documented on `AdrId`, so
    /// this exists solely for `..AdrRecord::test_sentinel()` struct-update
    /// syntax in test fixtures, never as a production construction path.
    pub(crate) fn test_sentinel() -> Self {
        Self {
            id: AdrId {
                prefix: String::new(),
                number: 0,
            },
            file_path: PathBuf::new(),
            title: None,
            title_line: 0,
            date: None,
            last_reviewed: None,
            tier: TierField::Absent,
            status: None,
            status_line: 0,
            status_raw: None,
            related: Related::Absent,
            has_context: false,
            has_decision: false,
            has_consequences: false,
            has_retirement: false,
            is_stale: false,
            status_from_section: false,
            max_code_block_lines: 0,
            max_code_block_line: 0,
            section_order: Vec::new(),
            section_word_counts: HashMap::new(),
            crates: Vec::new(),
            decision_rules: Vec::new(),
            parent_cross_domain: CrossDomainParent::Absent,
        }
    }
}

/// ADR tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    S,
    A,
    B,
    C,
    D,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S => f.write_str("S"),
            Self::A => f.write_str("A"),
            Self::B => f.write_str("B"),
            Self::C => f.write_str("C"),
            Self::D => f.write_str("D"),
        }
    }
}

impl Tier {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "S" => Some(Self::S),
            "A" => Some(Self::A),
            "B" => Some(Self::B),
            "C" => Some(Self::C),
            "D" => Some(Self::D),
            _ => None,
        }
    }

    /// Human-readable tier name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::S => "Intent",
            Self::A => "Self-organization",
            Self::B => "Design",
            Self::C => "Feedbacks",
            Self::D => "Parameters",
        }
    }

    /// Tier meaning and scope description.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::S => {
                "Paradigm, goals, or governance — changing reshapes the \
                        system's purpose and every tier below it."
            }
            Self::A => {
                "Extension points and structural evolvability — changing \
                        alters what the system can become."
            }
            Self::B => {
                "Type contracts, API boundaries, and information flows — \
                        changing requires coordinated updates across crates."
            }
            Self::C => {
                "Runtime behaviour and interaction dynamics — changing \
                        requires coordinated call-site updates."
            }
            Self::D => {
                "Implementation details and tooling configuration — \
                        changing affects only crate internals."
            }
        }
    }

    /// Stability expectation for this tier.
    #[must_use]
    pub fn stability(self) -> &'static str {
        match self {
            Self::S => "Immutable post-1.0",
            Self::A => "Near-immutable; changes require RFC-level discussion",
            Self::B => "Stable; changes documented via git history",
            Self::C => "Stable; changes require integration testing",
            Self::D => "Mutable; may be superseded freely",
        }
    }

    /// All tier variants in order.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[Self::S, Self::A, Self::B, Self::C, Self::D]
    }

    /// Numeric rank for sorting (S=0, A=1, ... D=4).
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Self::S => 0,
            Self::A => 1,
            Self::B => 2,
            Self::C => 3,
            Self::D => 4,
        }
    }

    /// Tier-scaling factor for word count and rule limits.
    ///
    /// S-tier decisions are broad (paradigm-level) and get more room.
    /// D-tier decisions are narrow (parameters) and should be tighter.
    /// Applied as a multiplier to `max_words` and `max_rules` base values.
    #[must_use]
    pub fn factor(self) -> f64 {
        match self {
            Self::S => 1.5,
            Self::A => 1.2,
            Self::B => 1.0,
            Self::C => 0.8,
            Self::D => 0.6,
        }
    }

    /// Tier-scaled minimum word count for prose sections.
    ///
    /// Higher-tier ADRs need more substance; lower-tier can be brief.
    #[must_use]
    pub fn min_words(self) -> u64 {
        match self {
            Self::S => 15,
            Self::A => 12,
            Self::B => 10,
            Self::C | Self::D => 7,
        }
    }

    /// Tier-scaled maximum reference count (References: targets only).
    ///
    /// Root and Supersedes are structural, not content dependencies,
    /// and do not count toward the load limit.
    ///
    /// The curve is non-monotonic: C-tier peaks at 8 (feedback loops
    /// often coordinate many components) while D-tier drops to 5
    /// (parameter decisions should have narrow scope). S-tier is
    /// tightest at 3 — paradigm decisions reference few peers.
    #[must_use]
    pub fn max_refs(self) -> usize {
        match self {
            Self::S => 3,
            Self::A | Self::D => 5,
            Self::B => 7,
            Self::C => 8,
        }
    }
}

/// ADR lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Draft,
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    SupersededBy(AdrId),
    /// Status line could not be parsed into a known variant.
    Invalid(String),
}

impl Status {
    /// Parse a status line. Returns `Invalid` if unrecognized.
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let trimmed = line.trim();

        match trimmed {
            "Draft" => Self::Draft,
            "Proposed" => Self::Proposed,
            "Accepted" => Self::Accepted,
            "Deprecated" => Self::Deprecated,
            "Rejected" => Self::Rejected,
            s if s.starts_with("Superseded by ") => {
                let rest = &s["Superseded by ".len()..];
                match parse_adr_id(rest.trim()) {
                    Some(id) => Self::SupersededBy(id),
                    None => Self::Invalid(trimmed.to_owned()),
                }
            }
            _ => Self::Invalid(trimmed.to_owned()),
        }
    }

    /// Returns true if the raw status line has parenthetical content
    /// (e.g., `Accepted (note)`), which is not a valid status format.
    #[must_use]
    pub fn has_parenthetical(raw: &str) -> bool {
        let trimmed = raw.trim();
        trimmed.contains('(') && trimmed.contains(')')
    }

    /// Returns true for terminal lifecycle states: Rejected, Deprecated,
    /// Superseded. Terminal-state ADRs must be in the stale directory
    /// and have a `## Retirement` section.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Rejected | Self::Deprecated | Self::SupersededBy(_)
        )
    }

    /// Short display string for output formatting.
    #[must_use]
    pub fn short_display(&self) -> String {
        match self {
            Self::Draft => "Draft".into(),
            Self::Proposed => "Proposed".into(),
            Self::Accepted => "Accepted".into(),
            Self::Rejected => "Rejected".into(),
            Self::Deprecated => "Deprecated".into(),
            Self::SupersededBy(id) => format!("Superseded by {id}"),
            Self::Invalid(s) => s.clone(),
        }
    }
}

/// A typed, directional relationship between two ADRs.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub verb: RelVerb,
    pub target: AdrId,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum CrossDomainParent {
    Absent,
    Valid {
        id: AdrId,
        reason: String,
    },
    Malformed {
        raw: String,
        reason: CrossDomainDefect,
    },
}

impl CrossDomainParent {
    pub(crate) fn honoured_id(&self) -> Option<&AdrId> {
        match self {
            Self::Valid { id, .. } => Some(id),
            Self::Absent | Self::Malformed { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CrossDomainDefect {
    EmptyField,
    UnparseableId(String),
    MissingReason,
}

impl fmt::Display for CrossDomainDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField => write!(f, "the field is present but empty"),
            Self::UnparseableId(id) => write!(f, "`{id}` is not a PREFIX-NNNN ADR id"),
            Self::MissingReason => write!(
                f,
                "no reason given after the ID — AFM-0020:R3 suppresses L011 only for \
                 `PREFIX-NNNN — reason`"
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TierField {
    Absent,
    Valid(Tier),
    Invalid { raw: String },
}

impl TierField {
    pub(crate) fn value(&self) -> Option<Tier> {
        match self {
            Self::Valid(tier) => Some(*tier),
            Self::Absent | Self::Invalid { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Related {
    Absent,
    Parsed(Vec<Relationship>),
    Malformed {
        line: String,
        reason: MalformedReason,
        relationships: Vec<Relationship>,
    },
}

impl Related {
    pub(crate) fn relationships(&self) -> &[Relationship] {
        match self {
            Self::Parsed(v)
            | Self::Malformed {
                relationships: v, ..
            } => v,
            Self::Absent => &[],
        }
    }

    pub(crate) fn malformed_summary(&self) -> Option<String> {
        match self {
            Self::Malformed { line, reason, .. } => Some(format!("{reason} in `{line}`")),
            Self::Parsed(_) | Self::Absent => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MalformedReason {
    MissingSeparator,
    UnknownVerb(String),
    UnparseableTarget(String),
}

impl fmt::Display for MalformedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSeparator => write!(f, "missing `Verb: ` separator"),
            Self::UnknownVerb(v) => write!(f, "unrecognized verb `{v}`"),
            Self::UnparseableTarget(t) => write!(f, "unparseable target `{t}`"),
        }
    }
}

/// Relationship verb vocabulary.
///
/// Three permitted verbs:
/// - `References` — soft citation (citing → cited)
/// - `Supersedes` — replaces target entirely (newer → older)
/// - `Root` — self-reference marking this ADR as a tree root
///
/// Legacy verbs are retained so the parser can recognize them and
/// guidelines output can show migration paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelVerb {
    References,
    Supersedes,
    Root,

    DependsOn,
    Extends,
    Illustrates,
    ContrastsWith,
    ScopedBy,

    Informs,
    ExtendedBy,
    IllustratedBy,
    ReferencedBy,
    SupersededBy,
    Scopes,
}

impl RelVerb {
    /// True for legacy reverse verbs.
    #[must_use]
    pub fn is_reverse(self) -> bool {
        matches!(
            self,
            Self::Informs
                | Self::ExtendedBy
                | Self::IllustratedBy
                | Self::ReferencedBy
                | Self::SupersededBy
                | Self::Scopes
        )
    }

    /// Human-readable description of the verb's meaning.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::Root => "Self-reference marking this ADR as a tree root",
            Self::References => "This ADR cites the target in context or consequences",
            Self::Supersedes => "Replaces target entirely; target becomes Deprecated/Superseded",
            _ => "Legacy verb — migrate to a permitted verb",
        }
    }

    /// Migration guidance for legacy verbs. Returns None for permitted verbs.
    #[must_use]
    pub fn migration(self) -> Option<&'static str> {
        match self {
            Self::DependsOn
            | Self::Extends
            | Self::Illustrates
            | Self::ContrastsWith
            | Self::ScopedBy => Some("use References"),
            Self::Informs
            | Self::ExtendedBy
            | Self::IllustratedBy
            | Self::ReferencedBy
            | Self::SupersededBy
            | Self::Scopes => Some("remove (reverse verb)"),
            _ => None,
        }
    }

    /// All permitted verb variants.
    #[must_use]
    pub fn permitted() -> &'static [Self] {
        &[Self::Root, Self::References, Self::Supersedes]
    }

    /// All legacy verb variants.
    #[must_use]
    pub fn legacy() -> &'static [Self] {
        &[
            Self::DependsOn,
            Self::Extends,
            Self::Illustrates,
            Self::ContrastsWith,
            Self::ScopedBy,
            Self::Informs,
            Self::ExtendedBy,
            Self::IllustratedBy,
            Self::ReferencedBy,
            Self::SupersededBy,
            Self::Scopes,
        ]
    }

    /// Parse a verb string from the `## Related` section.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "Root" => Some(Self::Root),
            "References" => Some(Self::References),
            "Supersedes" => Some(Self::Supersedes),
            "Depends on" => Some(Self::DependsOn),
            "Informs" => Some(Self::Informs),
            "Extends" => Some(Self::Extends),
            "Extended by" => Some(Self::ExtendedBy),
            "Illustrates" => Some(Self::Illustrates),
            "Illustrated by" => Some(Self::IllustratedBy),
            "Referenced by" => Some(Self::ReferencedBy),
            "Contrasts with" => Some(Self::ContrastsWith),
            "Superseded by" => Some(Self::SupersededBy),
            "Scopes" => Some(Self::Scopes),
            "Scoped by" => Some(Self::ScopedBy),
            _ => None,
        }
    }
}

impl fmt::Display for RelVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Root => "Root",
            Self::References => "References",
            Self::Supersedes => "Supersedes",
            Self::DependsOn => "Depends on",
            Self::Informs => "Informs",
            Self::Extends => "Extends",
            Self::ExtendedBy => "Extended by",
            Self::Illustrates => "Illustrates",
            Self::IllustratedBy => "Illustrated by",
            Self::ReferencedBy => "Referenced by",
            Self::ContrastsWith => "Contrasts with",
            Self::SupersededBy => "Superseded by",
            Self::Scopes => "Scopes",
            Self::ScopedBy => "Scoped by",
        };
        write!(f, "{s}")
    }
}

/// Parse a strict ADR ID like `CHE-0042`.
///
/// Accepts exactly `^[A-Z]{2,4}-[0-9]{4}$` — uppercase ASCII prefix
/// of 2–4 letters, dash, exactly 4 digits, nothing else. No
/// whitespace trimming; callers must pass clean input.
///
/// Returns `None` for any deviation: lowercase, non-ASCII, wrong
/// digit count, trailing text, leading/trailing whitespace.
///
/// Use [`parse_adr_id_from_filename_stem`] when the input is an
/// ADR filename stem like `CHE-0042-slug-words`.
///
/// Implemented with byte-level checks rather than regex per
/// AFM-0006 R1 (regex is reserved for markdown structural
/// extraction; lexical token validation may use byte checks).
#[must_use]
pub fn parse_adr_id(s: &str) -> Option<AdrId> {
    let (prefix, num_str) = s.split_once('-')?;

    let prefix_len = prefix.len();
    if !(2..=4).contains(&prefix_len) || !prefix.bytes().all(|b| b.is_ascii_uppercase()) {
        return None;
    }

    if num_str.len() != 4 || !num_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let number: u16 = num_str.parse().ok()?;

    Some(AdrId {
        prefix: prefix.to_owned(),
        number,
    })
}

/// Parse an ADR ID from a filename stem like `CHE-0042-some-slug`.
///
/// Matches `^[A-Z]{2,4}-[0-9]{4}(?:-|$)` at the start of the stem
/// and ignores everything after the trailing dash. Returns `None`
/// if the stem does not begin with a strict ID followed by either
/// a dash or end-of-string.
///
/// Specifically rejects `CHE-00012-foo` (5 digits before the dash)
/// and `CHE-0001x` (no dash separator after digits) — both of
/// which a naive prefix-match would silently accept.
///
/// The stem is the filename with `.md` already stripped by the
/// caller. Whitespace is not trimmed.
///
/// See AFM-0006 R1 for the byte-level validation rationale.
#[must_use]
pub fn parse_adr_id_from_filename_stem(stem: &str) -> Option<AdrId> {
    let (prefix, rest) = stem.split_once('-')?;

    let prefix_len = prefix.len();
    if !(2..=4).contains(&prefix_len) || !prefix.bytes().all(|b| b.is_ascii_uppercase()) {
        return None;
    }

    let rest_bytes = rest.as_bytes();
    if rest_bytes.len() < 4 || !rest_bytes[..4].iter().all(u8::is_ascii_digit) {
        return None;
    }

    if rest_bytes.len() > 4 && rest_bytes[4] != b'-' {
        return None;
    }

    let number: u16 = rest[..4].parse().ok()?;

    Some(AdrId {
        prefix: prefix.to_owned(),
        number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_adr_id_strict() {
        let id = parse_adr_id("CHE-0042").unwrap();
        assert_eq!(id.prefix, "CHE");
        assert_eq!(id.number, 42);
        assert_eq!(id.to_string(), "CHE-0042");
    }

    #[test]
    fn parse_adr_id_accepts_2_to_4_letter_prefix() {
        assert!(parse_adr_id("AI-0001").is_some());
        assert!(parse_adr_id("CHE-0001").is_some());
        assert!(parse_adr_id("AFRM-0001").is_some());
    }

    #[test]
    fn parse_adr_id_rejects_trailing_text() {
        assert!(parse_adr_id("CHE-0042-foo").is_none());
        assert!(parse_adr_id("CHE-0042 ").is_none());
    }

    #[test]
    fn parse_adr_id_rejects_whitespace() {
        assert!(parse_adr_id(" CHE-0042").is_none());
        assert!(parse_adr_id("CHE-0042\n").is_none());
    }

    #[test]
    fn parse_adr_id_rejects_non_ascii_prefix() {
        assert!(parse_adr_id("ÄDR-0001").is_none());
    }

    #[test]
    fn parse_adr_id_rejects_lowercase_prefix() {
        assert!(parse_adr_id("che-0001").is_none());
    }

    #[test]
    fn parse_adr_id_rejects_wrong_digit_count() {
        assert!(parse_adr_id("CHE-001").is_none());
        assert!(parse_adr_id("CHE-00001").is_none());
        assert!(parse_adr_id("CHE-").is_none());
    }

    #[test]
    fn parse_adr_id_rejects_short_or_long_prefix() {
        assert!(parse_adr_id("C-0001").is_none());
        assert!(parse_adr_id("ABCDE-0001").is_none());
    }

    #[test]
    fn parse_adr_id_empty_prefix_returns_none() {
        assert!(parse_adr_id("-0001").is_none());
        assert!(parse_adr_id("").is_none());
        assert!(parse_adr_id("-").is_none());
    }

    #[test]
    fn parse_adr_id_from_filename_stem_strips_slug() {
        let id = parse_adr_id_from_filename_stem("CHE-0042-some-slug-words").unwrap();
        assert_eq!(id.prefix, "CHE");
        assert_eq!(id.number, 42);
    }

    #[test]
    fn parse_adr_id_from_filename_stem_accepts_bare_id() {
        let id = parse_adr_id_from_filename_stem("CHE-0001").unwrap();
        assert_eq!(id.number, 1);
    }

    #[test]
    fn parse_adr_id_from_filename_stem_rejects_short_digits() {
        assert!(parse_adr_id_from_filename_stem("CHE-42-slug").is_none());
        assert!(parse_adr_id_from_filename_stem("CHE-001").is_none());
    }

    #[test]
    fn parse_adr_id_from_filename_stem_rejects_lowercase() {
        assert!(parse_adr_id_from_filename_stem("che-0001-slug").is_none());
    }

    #[test]
    fn parse_adr_id_from_filename_stem_rejects_non_ascii() {
        assert!(parse_adr_id_from_filename_stem("ÄDR-0001-slug").is_none());
    }

    #[test]
    fn parse_adr_id_from_filename_stem_rejects_five_digits() {
        assert!(parse_adr_id_from_filename_stem("CHE-00012-foo").is_none());
    }

    #[test]
    fn parse_adr_id_from_filename_stem_rejects_non_dash_separator() {
        assert!(parse_adr_id_from_filename_stem("CHE-0001x").is_none());
        assert!(parse_adr_id_from_filename_stem("CHE-0001_slug").is_none());
    }

    #[test]
    fn parse_status_superseded_with_trailing_space_returns_invalid() {
        let s = Status::parse("Superseded by ");
        assert!(matches!(s, Status::Invalid(_)), "got {s:?}");
    }

    #[test]
    fn parse_status_accepted() {
        assert_eq!(Status::parse("Accepted"), Status::Accepted);
    }

    #[test]
    fn parse_status_rejected() {
        assert_eq!(Status::parse("Rejected"), Status::Rejected);
    }

    #[test]
    fn parse_status_amended_is_invalid() {
        let s = Status::parse("Amended 2026-04-25 — added fencing");
        assert!(matches!(s, Status::Invalid(_)));
    }

    #[test]
    fn parse_status_amended_bare_is_invalid() {
        let s = Status::parse("Amended");
        assert!(matches!(s, Status::Invalid(_)));
    }

    #[test]
    fn parse_status_superseded() {
        let s = Status::parse("Superseded by CHE-0099");
        match s {
            Status::SupersededBy(id) => {
                assert_eq!(id.prefix, "CHE");
                assert_eq!(id.number, 99);
            }
            other => panic!("expected SupersededBy, got {other:?}"),
        }
    }

    #[test]
    fn parse_status_invalid() {
        let s = Status::parse("Accepted (supersedes original u64 design)");
        assert!(matches!(s, Status::Invalid(_)));
    }

    #[test]
    fn has_parenthetical_detects_annotations() {
        assert!(Status::has_parenthetical("Accepted (note)"));
        assert!(!Status::has_parenthetical("Accepted"));
        assert!(!Status::has_parenthetical("Amended 2026-04-25 — note"));
    }

    #[test]
    fn root_verb_parse_and_display() {
        assert_eq!(RelVerb::parse("Root"), Some(RelVerb::Root));
        assert_eq!(RelVerb::Root.to_string(), "Root");
    }

    #[test]
    fn verb_display_roundtrip() {
        let verbs = [
            ("Root", RelVerb::Root),
            ("References", RelVerb::References),
            ("Supersedes", RelVerb::Supersedes),
            ("Depends on", RelVerb::DependsOn),
            ("Informs", RelVerb::Informs),
            ("Extends", RelVerb::Extends),
            ("Extended by", RelVerb::ExtendedBy),
            ("Illustrates", RelVerb::Illustrates),
            ("Illustrated by", RelVerb::IllustratedBy),
            ("Referenced by", RelVerb::ReferencedBy),
            ("Contrasts with", RelVerb::ContrastsWith),
            ("Superseded by", RelVerb::SupersededBy),
            ("Scopes", RelVerb::Scopes),
            ("Scoped by", RelVerb::ScopedBy),
        ];
        for (text, verb) in verbs {
            assert_eq!(RelVerb::parse(text), Some(verb), "parse({text})");
            assert_eq!(verb.to_string(), text, "display({verb:?})");
        }
    }

    #[test]
    fn tier_descriptions_non_empty() {
        for tier in Tier::all() {
            assert!(!tier.name().is_empty(), "{tier:?} name");
            assert!(!tier.description().is_empty(), "{tier:?} description");
            assert!(!tier.stability().is_empty(), "{tier:?} stability");
        }
    }

    #[test]
    fn tier_names_match_meadows_alignment() {
        assert_eq!(Tier::S.name(), "Intent");
        assert_eq!(Tier::A.name(), "Self-organization");
        assert_eq!(Tier::B.name(), "Design");
        assert_eq!(Tier::C.name(), "Feedbacks");
        assert_eq!(Tier::D.name(), "Parameters");
    }

    #[test]
    fn status_is_terminal() {
        assert!(!Status::Draft.is_terminal());
        assert!(!Status::Proposed.is_terminal());
        assert!(!Status::Accepted.is_terminal());
        assert!(Status::Rejected.is_terminal());
        assert!(Status::Deprecated.is_terminal());
        assert!(
            Status::SupersededBy(AdrId {
                prefix: "CHE".into(),
                number: 1
            })
            .is_terminal()
        );
    }

    #[test]
    fn verb_migration_for_legacy() {
        for verb in RelVerb::legacy() {
            assert!(
                verb.migration().is_some(),
                "{verb:?} should have migration guidance"
            );
        }
    }

    #[test]
    fn verb_migration_none_for_permitted() {
        for verb in RelVerb::permitted() {
            assert!(
                verb.migration().is_none(),
                "{verb:?} should not have migration guidance"
            );
        }
    }

    #[test]
    fn sentinel_adr_record() {
        let record = AdrRecord::test_sentinel();
        assert_eq!(record.id.prefix, "");
        assert_eq!(record.id.number, 0);
        assert!(record.crates.is_empty());
        assert!(record.decision_rules.is_empty());
    }

    #[test]
    fn tier_rank_ordering() {
        assert!(Tier::S.rank() < Tier::A.rank());
        assert!(Tier::A.rank() < Tier::B.rank());
        assert!(Tier::B.rank() < Tier::C.rank());
        assert!(Tier::C.rank() < Tier::D.rank());
        assert_eq!(Tier::D.rank(), 4);
    }

    #[test]
    fn tier_factor_ordering() {
        assert!(Tier::S.factor() > Tier::A.factor());
        assert!(Tier::A.factor() > Tier::B.factor());
        assert!((Tier::B.factor() - 1.0).abs() < f64::EPSILON);
        assert!(Tier::C.factor() < Tier::B.factor());
        assert!(Tier::D.factor() < Tier::C.factor());
    }

    #[test]
    fn tier_min_words_ordering() {
        assert!(Tier::S.min_words() >= Tier::A.min_words());
        assert!(Tier::A.min_words() >= Tier::B.min_words());
        assert!(Tier::B.min_words() >= Tier::C.min_words());
        assert!(Tier::C.min_words() >= Tier::D.min_words());
    }

    #[test]
    fn tier_max_refs_values() {
        assert_eq!(Tier::S.max_refs(), 3);
        assert_eq!(Tier::A.max_refs(), 5);
        assert_eq!(Tier::B.max_refs(), 7);
        assert_eq!(Tier::C.max_refs(), 8);
        assert_eq!(Tier::D.max_refs(), 5);
    }

    #[test]
    fn status_short_display() {
        assert_eq!(Status::Draft.short_display(), "Draft");
        assert_eq!(Status::Accepted.short_display(), "Accepted");
        assert_eq!(
            Status::SupersededBy(AdrId {
                prefix: "CHE".into(),
                number: 99,
            })
            .short_display(),
            "Superseded by CHE-0099"
        );
    }

    #[test]
    fn layer_to_tier_mapping() {
        use super::layer_to_tier;

        assert_eq!(layer_to_tier(1), Some(Tier::S));
        assert_eq!(layer_to_tier(2), Some(Tier::S));
        assert_eq!(layer_to_tier(3), Some(Tier::S));

        assert_eq!(layer_to_tier(4), Some(Tier::A));

        assert_eq!(layer_to_tier(5), Some(Tier::B));
        assert_eq!(layer_to_tier(6), Some(Tier::B));

        assert_eq!(layer_to_tier(7), Some(Tier::C));
        assert_eq!(layer_to_tier(8), Some(Tier::C));

        assert_eq!(layer_to_tier(9), Some(Tier::D));
        assert_eq!(layer_to_tier(10), Some(Tier::D));
        assert_eq!(layer_to_tier(11), Some(Tier::D));
        assert_eq!(layer_to_tier(12), Some(Tier::D));
    }

    #[test]
    fn layer_to_tier_invalid() {
        use super::layer_to_tier;

        assert_eq!(layer_to_tier(0), None);
        assert_eq!(layer_to_tier(13), None);
        assert_eq!(layer_to_tier(255), None);
    }

    #[test]
    fn adr_id_try_new_accepts_valid_input() {
        let id = AdrId::try_new("CHE", 42).unwrap();
        assert_eq!(id.prefix(), "CHE");
        assert_eq!(id.number(), 42);
        assert_eq!(id.to_string(), "CHE-0042");
    }

    #[test]
    fn adr_id_try_new_rejects_short_prefix() {
        let err = AdrId::try_new("C", 1).unwrap_err();
        assert!(
            matches!(err, AdrIdError::PrefixLength { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn adr_id_try_new_rejects_long_prefix() {
        let err = AdrId::try_new("ABCDE", 1).unwrap_err();
        assert!(
            matches!(err, AdrIdError::PrefixLength { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn adr_id_try_new_rejects_lowercase_prefix() {
        let err = AdrId::try_new("che", 1).unwrap_err();
        assert!(
            matches!(err, AdrIdError::PrefixNotUppercaseAscii { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn adr_id_try_new_rejects_out_of_range_number() {
        let err = AdrId::try_new("CHE", 10_000).unwrap_err();
        assert!(
            matches!(err, AdrIdError::NumberOutOfRange { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn adr_id_try_from_str_round_trips() {
        let id = AdrId::try_from("CHE-0042").unwrap();
        assert_eq!(id.prefix(), "CHE");
        assert_eq!(id.number(), 42);
    }

    #[test]
    fn adr_id_try_from_str_rejects_malformed() {
        let err = AdrId::try_from("CHE0042").unwrap_err();
        assert!(matches!(err, AdrIdError::Malformed { .. }), "got {err:?}");
    }

    #[test]
    fn adr_id_error_impls_error_trait() {
        let err = AdrId::try_new("c", 1).unwrap_err();
        let _: &dyn std::error::Error = &err;
        assert!(!err.to_string().is_empty());
    }
}
