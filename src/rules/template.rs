//! Template compliance rules (T002–T022) and structure rules (S004–S008).
//!
//! T002–T005: Date/Last-reviewed/Tier/Status metadata; T005c: legacy `## Status`.
//! T006: status validity. T007–T010: required active sections. T011: code-block
//! size. T014: section order. T015: tier-scaled prose range. T016: tagged
//! rules (tier-scaled count, 7–60 words each). T019: rule/ADR tier tension.
//! T020: tier-scaled `References:` load. T022: omitted-MADR-section residue
//! in active ADRs.
//!
//! S004/S005: stale/Retirement mismatch. S006: terminal ADR outside stale.
//! S007: disallowed stale-stub sections or non-lineage verbs per AFM-0022.
//! S008: live-status ADR inside stale (S006's reverse).

use crate::rules::catalog;
use std::path::Path;

use crate::config::{Config, RuleParam};
use crate::model::{
    AdrRecord, DateError, DateField, LiveStatus, RelVerb, Related, Status, StatusLiveness,
    TaggedRule, Tier, TierField, layer_to_tier,
};
use crate::report::Diagnostic;

const MAX_CODE_BLOCK_LINES: usize = 20;

const DEFAULT_MIN_WORDS: u64 = 7;

const DEFAULT_MAX_WORDS: u64 = 100;

const DEFAULT_MAX_RULES: u64 = 10;

const DEFAULT_MIN_RULE_WORDS: u64 = 7;

const DEFAULT_MAX_RULE_WORDS: u64 = 60;

const ACTIVE_SECTION_ORDER_WITH_STATUS: &[&str] =
    &["Status", "Related", "Context", "Decision", "Consequences"];

const ACTIVE_SECTION_ORDER: &[&str] = &["Related", "Context", "Decision", "Consequences"];

const STALE_SECTION_ORDER_WITH_STATUS: &[&str] = &[
    "Status",
    "Related",
    "Context",
    "Decision",
    "Consequences",
    "Retirement",
];

const STALE_SECTION_ORDER: &[&str] = &[
    "Related",
    "Context",
    "Decision",
    "Consequences",
    "Retirement",
];

const STUB_ALLOWED_SECTIONS: &[&str] = &["Related", "Retirement"];

const CONFIG_FILE_NAME: &str = "adr-fmt.toml";

pub(crate) struct Budgets {
    max_words: u64,
    min_words: u64,
    max_rules: u64,
    min_rule_words: u64,
    max_rule_words: u64,
}

impl Budgets {
    pub(crate) fn resolve(config: &Config, diags: &mut Vec<Diagnostic>) -> Self {
        Self {
            max_words: resolve_param(
                config,
                &catalog::T015,
                "max_words",
                DEFAULT_MAX_WORDS,
                diags,
            ),
            min_words: resolve_param(
                config,
                &catalog::T015,
                "min_words",
                DEFAULT_MIN_WORDS,
                diags,
            ),
            max_rules: resolve_param(
                config,
                &catalog::T016,
                "max_rules",
                DEFAULT_MAX_RULES,
                diags,
            ),
            min_rule_words: resolve_param(
                config,
                &catalog::T016,
                "min_rule_words",
                DEFAULT_MIN_RULE_WORDS,
                diags,
            ),
            max_rule_words: resolve_param(
                config,
                &catalog::T016,
                "max_rule_words",
                DEFAULT_MAX_RULE_WORDS,
                diags,
            ),
        }
    }

    pub(crate) fn min_words_for(&self, tier: Tier) -> u64 {
        tier_scaled(self.min_words, tier)
    }
}

fn resolve_param(
    config: &Config,
    rule: &'static catalog::RuleEntry,
    key: &str,
    default: u64,
    diags: &mut Vec<Diagnostic>,
) -> u64 {
    match config.rule_param_u64(rule.id, key) {
        RuleParam::Value(value) => value,
        RuleParam::Absent => default,
        RuleParam::Invalid {
            rule_id,
            key,
            reason,
        } => {
            diags.push(rule.diagnostic(
                Path::new(CONFIG_FILE_NAME),
                0,
                format!(
                    "[[rules]] id = \"{rule_id}\" parameter `{key}` is not usable: \
                     {reason}. Action: fix the value in {CONFIG_FILE_NAME}; until \
                     then the built-in default {default} applies and validation \
                     does not reflect the configured policy."
                ),
            ));
            default
        }
    }
}

mod tier_lane {
    use crate::model::{Tier, TierField};

    #[derive(Clone, Copy)]
    pub(super) struct ValidTier(Tier);

    impl ValidTier {
        pub(super) fn get(self) -> Tier {
            self.0
        }
    }

    impl std::fmt::Display for ValidTier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Clone, Copy)]
    pub(super) enum TierScaling {
        Scaled(ValidTier),
        Indeterminate,
    }

    impl TierScaling {
        pub(super) fn classify(field: &TierField) -> Self {
            match field {
                TierField::Valid(tier) => Self::Scaled(ValidTier(*tier)),
                TierField::Absent | TierField::Invalid { .. } => Self::Indeterminate,
            }
        }
    }
}

use tier_lane::{TierScaling, ValidTier};

const TIER_SCALED_CHECKS: &str = "tier-scaled checks (T015 word budgets, T016 rule-count \
                                  budget, T019 tier tension, T020 reference load) cannot be \
                                  evaluated and were skipped";

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "rounding a non-negative bounded tier-scaled budget to u64; value is small and non-negative by construction, truncation/sign-loss cannot occur"
)]
fn tier_scaled(base: u64, tier: Tier) -> u64 {
    (base as f64 * tier.factor()).round() as u64
}

fn scale(base: u64, tier: ValidTier) -> u64 {
    tier_scaled(base, tier.get())
}

pub fn check(record: &AdrRecord, config: &Config, budgets: &Budgets, diags: &mut Vec<Diagnostic>) {
    check_metadata(record, diags);
    check_status_validity(record, diags);
    check_structure(record, diags);
    check_section_order(record, diags);
    check_residue_sections(record, diags);

    check_tagged_rules(
        record,
        budgets.min_rule_words,
        budgets.max_rule_words,
        diags,
    );

    let effective_min = match TierScaling::classify(record.tier_field()) {
        TierScaling::Scaled(tier) => {
            let min_words = budgets.min_words_for(tier.get());
            let max_words = scale(budgets.max_words, tier);
            check_section_word_counts(record, min_words, max_words, tier, diags);
            check_rule_count(record, tier, scale(budgets.max_rules, tier), diags);
            check_rule_tier_tension(record, tier, diags);
            check_reference_load(record, tier, diags);
            Some(min_words)
        }
        TierScaling::Indeterminate => None,
    };

    check_stale_lifecycle(record, config, effective_min, diags);
    check_stale_stub_structure(record, diags);
}

fn malformed_date_message(label: &str, raw: &str, reason: DateError) -> String {
    format!(
        "`{label}` value `{}` is not a `YYYY-MM-DD` date — {reason}",
        raw.escape_debug()
    )
}

fn check_date_fields(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    match record.date_field() {
        DateField::Valid { .. } => {}
        DateField::Absent => {
            diags.push(catalog::T002.diagnostic(
                record.file_path(),
                0,
                "missing `Date:` field".into(),
            ));
        }
        DateField::Invalid { raw, reason } => {
            diags.push(catalog::T023.diagnostic(
                record.file_path(),
                0,
                malformed_date_message("Date:", raw, *reason),
            ));
        }
    }

    match record.last_reviewed_field() {
        DateField::Valid { .. } => {}
        DateField::Absent => {
            diags.push(catalog::T003.diagnostic(
                record.file_path(),
                0,
                "missing `Last-reviewed:` field (required for all tiers)".into(),
            ));
        }
        DateField::Invalid { raw, reason } => {
            diags.push(catalog::T023.diagnostic(
                record.file_path(),
                0,
                malformed_date_message("Last-reviewed:", raw, *reason),
            ));
        }
    }
}

fn check_metadata(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    check_date_fields(record, diags);

    match record.tier_field() {
        TierField::Valid(_) => {}
        TierField::Absent => {
            diags.push(catalog::T004.diagnostic(
                record.file_path(),
                0,
                format!("missing `Tier:` field — {TIER_SCALED_CHECKS}"),
            ));
        }
        TierField::Invalid { raw } => {
            diags.push(catalog::T004.diagnostic(
                record.file_path(),
                0,
                format!(
                    "unrecognized `Tier:` value `{}` — expected one of S/A/B/C/D; \
                     {TIER_SCALED_CHECKS}",
                    raw.escape_debug()
                ),
            ));
        }
    }

    if record.status().is_none() {
        diags.push(
            catalog::T005.diagnostic(
                record.file_path(),
                0,
                "no status value — add a `Status:` preamble metadata field \
             (e.g., `Status: Accepted`)"
                    .into(),
            ),
        );
    }

    if let Some(heading_line) = record.legacy_status_section_line() {
        let message = match (record.status().is_some(), record.status_from_section()) {
            (true, true) => "status uses legacy `## Status` section — migrate to \
                 `Status:` preamble metadata field (e.g., `Status: Accepted`)"
                .to_owned(),
            (true, false) => "legacy `## Status` section is dead content — delete the leftover \
                 section; the `Status:` preamble metadata field is authoritative"
                .to_owned(),
            (false, _) => "legacy `## Status` section carries no status value — delete the \
                 empty section and add a `Status:` preamble metadata field \
                 (e.g., `Status: Accepted`)"
                .to_owned(),
        };
        diags.push(catalog::T005C.diagnostic(record.file_path(), heading_line, message));
    }
}

enum InvalidStatus<'a> {
    Parenthetical(&'a str),
    Unrecognized(&'a str),
}

impl<'a> InvalidStatus<'a> {
    fn classify(raw: &'a str, status: Option<&'a Status>) -> Option<Self> {
        match status {
            Some(Status::Invalid(_)) if Status::has_parenthetical(raw) => {
                Some(Self::Parenthetical(raw))
            }
            Some(Status::Invalid(s)) => Some(Self::Unrecognized(s)),
            _ => None,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Parenthetical(raw) => format!(
                "status line contains parenthetical annotation: `{raw}` — \
                 remove annotations, use a valid status keyword"
            ),
            Self::Unrecognized(s) => format!(
                "unrecognized status: `{s}` — expected one of: \
                 Draft, Proposed, Accepted, Rejected, Deprecated, \
                 Superseded by PREFIX-NNNN"
            ),
        }
    }
}

fn check_status_validity(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if let Some(raw) = record.status_raw()
        && let Some(invalid) = InvalidStatus::classify(raw, record.status())
    {
        diags.push(catalog::T006.diagnostic(
            record.file_path(),
            record.status_line(),
            invalid.message(),
        ));
    }

    if record.is_stale() {
        return;
    }
    if matches!(record.related(), Related::Absent) {
        diags.push(catalog::T007.diagnostic(
            record.file_path(),
            0,
            "missing `## Related` section".into(),
        ));
    } else if record.relationships().is_empty() {
        let message = if let Some(summary) = record.related().malformed_summary() {
            format!(
                "Related section has no relationships — every ADR must \
                 have at least one relation (use `Root: ID` for tree roots) \
                 (P003 already reported: {summary})"
            )
        } else {
            "Related section has no relationships — every ADR must \
             have at least one relation (use `Root: ID` for tree roots)"
                .to_owned()
        };
        diags.push(catalog::T007.diagnostic(record.file_path(), 0, message));
    }
}

fn check_structure(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if !record.is_stale() && !record.has_context() {
        diags.push(catalog::T008.diagnostic(
            record.file_path(),
            0,
            "missing `## Context` section".into(),
        ));
    }

    if !record.is_stale() && !record.has_decision() {
        diags.push(catalog::T009.diagnostic(
            record.file_path(),
            0,
            "missing `## Decision` section".into(),
        ));
    }

    if !record.is_stale() && !record.has_consequences() {
        diags.push(catalog::T010.diagnostic(
            record.file_path(),
            0,
            "missing `## Consequences` section".into(),
        ));
    }

    if record.max_code_block_lines() > MAX_CODE_BLOCK_LINES {
        diags.push(catalog::T011.diagnostic(
            record.file_path(),
            record.max_code_block_line(),
            format!(
                "code block has {} lines (max {}). \
                 Use signatures or pseudocode; reference source files \
                 for full implementations.",
                record.max_code_block_lines(),
                MAX_CODE_BLOCK_LINES,
            ),
        ));
    }
}

#[derive(Clone, Copy)]
enum RetirementPlacement {
    StaleWithoutRetirement,
    ActiveWithRetirement,
    Consistent,
}

#[derive(Clone, Copy)]
enum StatusPlacement<'a> {
    TerminalOutsideStale(&'a Status),
    LiveInsideStale(LiveStatus),
    IndeterminateInsideStale,
    Consistent,
}

struct StaleLifecycleVerdict<'a> {
    retirement: RetirementPlacement,
    status: StatusPlacement<'a>,
}

impl<'a> StaleLifecycleVerdict<'a> {
    fn classify(record: &'a AdrRecord) -> Self {
        let is_stale = record.is_stale();
        let has_retirement = record.has_retirement();

        let retirement = if is_stale && !has_retirement {
            RetirementPlacement::StaleWithoutRetirement
        } else if !is_stale && has_retirement {
            RetirementPlacement::ActiveWithRetirement
        } else {
            RetirementPlacement::Consistent
        };

        let status = match record.status() {
            None => StatusPlacement::Consistent,
            Some(status) => match (is_stale, StatusLiveness::classify(status)) {
                (false, StatusLiveness::Terminal) => StatusPlacement::TerminalOutsideStale(status),
                (true, StatusLiveness::Live(live)) => StatusPlacement::LiveInsideStale(live),
                (true, StatusLiveness::Indeterminate) => StatusPlacement::IndeterminateInsideStale,
                _ => StatusPlacement::Consistent,
            },
        };

        Self { retirement, status }
    }

    fn emit(
        &self,
        record: &AdrRecord,
        config: &Config,
        effective_min: Option<u64>,
        diags: &mut Vec<Diagnostic>,
    ) {
        match self.retirement {
            RetirementPlacement::StaleWithoutRetirement => diags.push(
                catalog::S004.diagnostic(
                    record.file_path(),
                    0,
                    "stale ADR missing `## Retirement` section — explain why \
                 this ADR was retired"
                        .into(),
                ),
            ),
            RetirementPlacement::ActiveWithRetirement => diags.push(
                catalog::S005.diagnostic(
                    record.file_path(),
                    0,
                    "active ADR has `## Retirement` section — Retirement is \
                 only for stale ADRs"
                        .into(),
                ),
            ),
            RetirementPlacement::Consistent => {}
        }

        match self.status {
            StatusPlacement::TerminalOutsideStale(status) => diags.push(catalog::S006.diagnostic(
                record.file_path(),
                record.status_line(),
                terminal_outside_stale_message(record, status, config, effective_min),
            )),
            StatusPlacement::LiveInsideStale(live) => diags.push(catalog::S008.diagnostic(
                record.file_path(),
                record.status_line(),
                live_inside_stale_message(record, live, config),
            )),
            StatusPlacement::IndeterminateInsideStale => {
                debug_assert!(
                    {
                        let file = record.file_path().display().to_string();
                        diags.iter().any(|d| d.rule == "T006" && d.file == file)
                    },
                    "an indeterminate status must already have been diagnosed by T006 \
                     before the stale lifecycle check declines to emit S008"
                );
            }
            StatusPlacement::Consistent => {}
        }
    }
}

fn terminal_outside_stale_message(
    record: &AdrRecord,
    status: &Status,
    config: &Config,
    effective_min: Option<u64>,
) -> String {
    let retirement_budget = match effective_min {
        Some(min_words) => format!(" (≥{min_words} words)"),
        None => String::new(),
    };
    format!(
        "{} has terminal status '{status}' but is not in the \
         stale directory. Action: move this file to {stale_dir}/ and add a \
         `## Retirement` section{retirement_budget} explaining why this \
         ADR left active service.",
        record.id(),
        stale_dir = config.stale.directory,
    )
}

fn live_inside_stale_message(record: &AdrRecord, live: LiveStatus, config: &Config) -> String {
    format!(
        "{} is in the {stale_dir}/ directory but has non-terminal status \
         '{live}'. Action: either set a terminal status (Rejected, \
         Deprecated, or Superseded by PREFIX-NNNN) to record how this ADR \
         left active service, or move the file back out of {stale_dir}/.",
        record.id(),
        stale_dir = config.stale.directory,
    )
}

fn check_stale_lifecycle(
    record: &AdrRecord,
    config: &Config,
    effective_min: Option<u64>,
    diags: &mut Vec<Diagnostic>,
) {
    StaleLifecycleVerdict::classify(record).emit(record, config, effective_min, diags);
}

fn check_stale_stub_structure(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if !record.is_stale() {
        return;
    }
    let Some(status) = record.status() else {
        return;
    };
    match StatusLiveness::classify(status) {
        StatusLiveness::Terminal => {}
        StatusLiveness::Live(_) => return,
        StatusLiveness::Indeterminate => {
            debug_assert!(
                {
                    let file = record.file_path().display().to_string();
                    diags.iter().any(|d| d.rule == "T006" && d.file == file)
                },
                "an indeterminate status must already have been diagnosed by T006 \
                 before the stale stub structure check declines to emit S007"
            );
            return;
        }
    }

    for section in record.section_order() {
        if !STUB_ALLOWED_SECTIONS.contains(&section.as_str()) {
            diags.push(catalog::S007.diagnostic(
                record.file_path(),
                0,
                format!(
                    "stale stub must not contain `## {section}` — \
                     terminal-state ADRs reduce to preamble + optional \
                     `## Related` (lineage edges only) + `## Retirement`. \
                     Delete this section; the prior content remains in git \
                     history. (See AFM-0022.)"
                ),
            ));
        }
    }

    for rel in record.relationships() {
        if !matches!(rel.verb, RelVerb::Supersedes) {
            diags.push(catalog::S007.diagnostic(
                record.file_path(),
                rel.line,
                format!(
                    "stale stub `## Related` must contain only `Supersedes:` \
                     edges (the reverse direction is recorded in `Status:`); \
                     found `{verb}: {target}`. Remove this edge — non-lineage \
                     citations on retired ADRs pollute the active reference \
                     graph. (See AFM-0022.)",
                    verb = rel.verb,
                    target = rel.target,
                ),
            ));
        }
    }
}

fn check_section_order(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    let has_status_section = record.section_order().iter().any(|s| s == "Status");

    let expected: &[&str] = match (record.is_stale(), has_status_section) {
        (true, true) => STALE_SECTION_ORDER_WITH_STATUS,
        (true, false) => STALE_SECTION_ORDER,
        (false, true) => ACTIVE_SECTION_ORDER_WITH_STATUS,
        (false, false) => ACTIVE_SECTION_ORDER,
    };

    let actual: Vec<&str> = record
        .section_order()
        .iter()
        .map(String::as_str)
        .filter(|s| expected.contains(s))
        .collect();

    let mut expected_iter = expected.iter();
    for actual_section in &actual {
        let mut found = false;
        for expected_section in expected_iter.by_ref() {
            if actual_section == expected_section {
                found = true;
                break;
            }
        }
        if !found {
            diags.push(catalog::T014.diagnostic(
                record.file_path(),
                0,
                format!(
                    "section `## {actual_section}` is out of canonical order — \
                     expected: {}",
                    expected.join(" → "),
                ),
            ));
            return;
        }
    }
}

const MADR_RESIDUE_SECTIONS: &[&str] = &[
    "context and problem statement",
    "decision drivers",
    "considered options",
    "options",
    "decision outcome",
    "pros and cons of the options",
    "pros and cons",
];

fn check_residue_sections(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if record.is_stale() {
        return;
    }

    for section in record.section_order() {
        if MADR_RESIDUE_SECTIONS.contains(&section.to_lowercase().as_str()) {
            diags.push(catalog::T022.diagnostic(
                record.file_path(),
                0,
                format!(
                    "`## {section}` is a MADR template section this template omits — \
                     fold its content into `## Context` or `## Decision` and remove \
                     the heading"
                ),
            ));
        }
    }
}

fn check_section_word_counts(
    record: &AdrRecord,
    min_words: u64,
    max_words: u64,
    tier: ValidTier,
    diags: &mut Vec<Diagnostic>,
) {
    let prose_sections = ["Context", "Consequences"];

    for section in &prose_sections {
        if let Some(&count) = record.section_word_counts().get(*section) {
            if (count as u64) < min_words {
                diags.push(catalog::T015.diagnostic(
                    record.file_path(),
                    0,
                    format!(
                        "`## {section}` has {count} word(s) ({tier}-tier minimum {min_words}) — \
                         provide more context"
                    ),
                ));
            } else if (count as u64) > max_words {
                diags.push(catalog::T015.diagnostic(
                    record.file_path(),
                    0,
                    format!(
                        "`## {section}` has {count} word(s) ({tier}-tier limit {max_words}) — \
                         consider tightening prose, splitting, or re-tiering"
                    ),
                ));
            }
        }
    }

    if record.has_retirement()
        && let Some(&count) = record.section_word_counts().get("Retirement")
    {
        if (count as u64) < min_words {
            diags.push(catalog::S004.diagnostic(
                record.file_path(),
                0,
                format!(
                    "`## Retirement` has {count} word(s) ({tier}-tier minimum {min_words}) — \
                     explain why this ADR was retired"
                ),
            ));
        } else if (count as u64) > max_words {
            diags.push(catalog::T015.diagnostic(
                record.file_path(),
                0,
                format!(
                    "`## Retirement` has {count} word(s) ({tier}-tier limit {max_words}) — \
                     be concise"
                ),
            ));
        }
    }
}

fn check_rule_count(
    record: &AdrRecord,
    tier: ValidTier,
    max_rules: u64,
    diags: &mut Vec<Diagnostic>,
) {
    if record.is_stale() || record.decision_rules().is_empty() {
        return;
    }
    if record.decision_rules().len() as u64 > max_rules {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            0,
            format!(
                "Decision section has {} tagged rules ({tier}-tier limit {max_rules}) — \
                 some tension is expected; consider splitting or re-tiering if scope is broad",
                record.decision_rules().len(),
            ),
        ));
    }
}

#[derive(PartialEq, Eq)]
enum RuleSetPresence {
    Present,
    Absent,
}

fn check_tagged_rules(
    record: &AdrRecord,
    min_rule_words: u64,
    max_rule_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    if record.is_stale() {
        return;
    }
    if check_rule_presence(record, diags) == RuleSetPresence::Absent {
        return;
    }
    check_rule_annotations(record, min_rule_words, max_rule_words, diags);
    check_rule_sequence(record, diags);
}

fn check_rule_presence(record: &AdrRecord, diags: &mut Vec<Diagnostic>) -> RuleSetPresence {
    for candidate in record.malformed_decision_rules() {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            candidate.line,
            format!(
                "Rule-shaped line does not match the required `RN [L]: text` \
                 format — `{}`",
                candidate.raw.escape_debug()
            ),
        ));
    }

    if record.decision_rules().is_empty() {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            0,
            "Decision section lacks tagged rules (RN [L]: pattern)".into(),
        ));
        return RuleSetPresence::Absent;
    }
    RuleSetPresence::Present
}

fn check_rule_annotations(
    record: &AdrRecord,
    min_rule_words: u64,
    max_rule_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    for rule in record.decision_rules() {
        check_rule_word_budget(record, rule, min_rule_words, max_rule_words, diags);
        check_rule_layer_range(record, rule, diags);
    }
}

fn check_rule_word_budget(
    record: &AdrRecord,
    rule: &TaggedRule,
    min_rule_words: u64,
    max_rule_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    let word_count = rule.text.split_whitespace().count() as u64;
    if word_count < min_rule_words {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            rule.line,
            format!(
                "Rule {id} has {word_count} word(s) (minimum {min_rule_words})",
                id = rule.id,
            ),
        ));
    } else if word_count > max_rule_words {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            rule.line,
            format!(
                "Rule {id} has {word_count} word(s) (maximum {max_rule_words}) — be concise",
                id = rule.id,
            ),
        ));
    }
}

fn check_rule_layer_range(record: &AdrRecord, rule: &TaggedRule, diags: &mut Vec<Diagnostic>) {
    if rule.layer == 0 || rule.layer > 12 {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            rule.line,
            format!(
                "Rule {id} has layer {layer} (must be 1-12, Meadows leverage points)",
                id = rule.id,
                layer = record.unparsed_rule_layer(rule.line).map_or_else(
                    || rule.layer.to_string(),
                    str::to_owned,
                ),
            ),
        ));
    }
}

fn check_rule_sequence(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    let mut nums: Vec<u32> = Vec::new();
    for rule in record.decision_rules() {
        if let Some(num) = rule.id.ordinal() {
            nums.push(num);
        }
    }

    nums.sort_unstable();
    if let Some([duplicate, _]) = nums.windows(2).find(|pair| pair[0] == pair[1]) {
        diags.push(catalog::T016.diagnostic(
            record.file_path(),
            0,
            format!("Duplicate tagged rule ID R{duplicate}"),
        ));
        return;
    }
    for (i, &num) in nums.iter().enumerate() {
        let expected = u32::try_from(i).expect("rule count fits u32") + 1;
        if num != expected {
            let prev = if i > 0 {
                format!("R{}", nums[i - 1])
            } else {
                "start".into()
            };
            diags.push(catalog::T016.diagnostic(
                record.file_path(),
                0,
                format!("Tagged rule IDs not sequential (gap after {prev})"),
            ));
            return;
        }
    }
}

fn check_rule_tier_tension(record: &AdrRecord, adr_tier: ValidTier, diags: &mut Vec<Diagnostic>) {
    let adr_rank = adr_tier.get().rank();

    for rule in record.decision_rules() {
        let Some(rule_tier) = layer_to_tier(rule.layer) else {
            continue;
        };
        let rule_rank = rule_tier.rank();
        if rule_rank < adr_rank {
            let distance = adr_rank - rule_rank;
            diags.push(catalog::T019.diagnostic(
                record.file_path(),
                rule.line,
                format!(
                    "Rule {} at layer {} ({rule_tier:?}-tier) is {distance} tiers \
                     from ADR tier {adr_tier} — tension may be intentional; \
                     consider adjusting layer, splitting rule to a {rule_tier:?}-tier ADR, \
                     or re-tiering this ADR",
                    rule.id, rule.layer,
                ),
            ));
        }
    }
}

fn check_reference_load(record: &AdrRecord, tier: ValidTier, diags: &mut Vec<Diagnostic>) {
    use crate::model::RelVerb;

    let ref_count = record
        .relationships()
        .iter()
        .filter(|r| r.verb == RelVerb::References)
        .count();

    let max_refs = tier.get().max_refs();
    if ref_count > max_refs {
        diags.push(catalog::T020.diagnostic(
            record.file_path(),
            0,
            format!(
                "{ref_count} references ({tier}-tier limit {max_refs}) — \
                 may indicate broad scope; consider splitting or promoting to a higher tier",
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Tier;
    use std::fmt::Write as _;

    const PROSE_WORDS: &[&str] = &[
        "every", "decision", "records", "a", "durable", "choice", "for", "the", "corpus", "and",
        "keeps", "its", "meaning", "stable", "over", "time",
    ];

    fn prose(words: usize) -> String {
        (0..words)
            .map(|i| PROSE_WORDS[i % PROSE_WORDS.len()])
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn line_of(body: &str, needle: &str) -> usize {
        body.lines()
            .position(|line| line.starts_with(needle))
            .map_or_else(
                || panic!("no line starts with `{needle}` in:\n{body}"),
                |index| index + 1,
            )
    }

    struct Doc {
        prefix: String,
        tier: Option<String>,
        status: Option<String>,
        date: Option<String>,
        last_reviewed: Option<String>,
        related: Option<String>,
        context: Option<String>,
        decision: Option<String>,
        consequences: Option<String>,
        extra: Vec<(String, String)>,
    }

    impl Doc {
        fn new() -> Self {
            Self {
                prefix: "CHE".to_owned(),
                tier: Some("S".to_owned()),
                status: Some("Accepted".to_owned()),
                date: Some("2026-04-25".to_owned()),
                last_reviewed: Some("2026-04-25".to_owned()),
                related: Some("Root: CHE-0001".to_owned()),
                context: Some(prose(15)),
                decision: Some(format!("R1 [5]: {}", prose(8))),
                consequences: Some(prose(15)),
                extra: Vec::new(),
            }
        }

        fn prefix(mut self, prefix: &str) -> Self {
            self.prefix = prefix.to_owned();
            self.related = Some(format!("Root: {prefix}-0001"));
            self
        }

        fn tier(mut self, tier: &str) -> Self {
            self.tier = Some(tier.to_owned());
            self
        }

        fn tier_line(self, tier: &str) -> Self {
            if tier.is_empty() {
                self.no_tier()
            } else {
                self.tier(tier)
            }
        }

        fn no_tier(mut self) -> Self {
            self.tier = None;
            self
        }

        fn status(mut self, status: &str) -> Self {
            self.status = Some(status.to_owned());
            self
        }

        fn no_status(mut self) -> Self {
            self.status = None;
            self
        }

        fn date(mut self, date: &str) -> Self {
            self.date = Some(date.to_owned());
            self
        }

        fn no_date(mut self) -> Self {
            self.date = None;
            self
        }

        fn last_reviewed(mut self, last_reviewed: &str) -> Self {
            self.last_reviewed = Some(last_reviewed.to_owned());
            self
        }

        fn no_last_reviewed(mut self) -> Self {
            self.last_reviewed = None;
            self
        }

        fn related(mut self, related: &str) -> Self {
            self.related = Some(related.to_owned());
            self
        }

        fn no_related(mut self) -> Self {
            self.related = None;
            self
        }

        fn context(mut self, context: &str) -> Self {
            self.context = Some(context.to_owned());
            self
        }

        fn no_context(mut self) -> Self {
            self.context = None;
            self
        }

        fn decision(mut self, decision: &str) -> Self {
            self.decision = Some(decision.to_owned());
            self
        }

        fn no_decision(mut self) -> Self {
            self.decision = None;
            self
        }

        fn consequences(mut self, consequences: &str) -> Self {
            self.consequences = Some(consequences.to_owned());
            self
        }

        fn no_consequences(mut self) -> Self {
            self.consequences = None;
            self
        }

        fn extra(mut self, heading: &str, section: &str) -> Self {
            self.extra.push((heading.to_owned(), section.to_owned()));
            self
        }

        fn body(self) -> String {
            let mut out = format!("# {}-0001. Test ADR\n\n", self.prefix);
            for (label, value) in [
                ("Date:", &self.date),
                ("Last-reviewed:", &self.last_reviewed),
                ("Tier:", &self.tier),
                ("Status:", &self.status),
            ] {
                if let Some(value) = value {
                    let _ = writeln!(out, "{label} {value}");
                }
            }
            out.push('\n');
            for (heading, section) in [
                ("Related", self.related),
                ("Context", self.context),
                ("Decision", self.decision),
                ("Consequences", self.consequences),
            ] {
                let Some(section) = section else { continue };
                let _ = write!(out, "## {heading}\n\n{section}\n\n");
            }
            for (heading, section) in self.extra {
                let _ = write!(out, "## {heading}\n\n{section}\n\n");
            }
            out
        }

        fn without_extra(mut self, heading: &str) -> Self {
            self.extra.retain(|(name, _)| name != heading);
            self
        }
    }

    fn numbered_rules(count: u16, layer: impl Fn(u16) -> u8) -> String {
        (1..=count)
            .map(|n| format!("R{n} [{}]: {}", layer(n), prose(8)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn references(count: u16) -> String {
        (2..2 + count)
            .map(|n| format!("References: CHE-{n:04}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn diagnostics_in_lane(
        config: &Config,
        prefix: &str,
        is_stale: bool,
        body: &str,
    ) -> Vec<Diagnostic> {
        let crate::parser::ParseFileOutcome::Parsed { record, .. } =
            crate::parser::parse_body(&format!("{prefix}-0001-body.md"), body, prefix, is_stale)
        else {
            panic!("test body did not yield a parsed record:\n{body}");
        };
        let mut diags = Vec::new();
        let budgets = Budgets::resolve(config, &mut diags);
        super::check(&record, config, &budgets, &mut diags);
        diags
    }

    fn diagnostics_for(body: &str) -> Vec<Diagnostic> {
        diagnostics_in_lane(&make_config(), "CHE", false, body)
    }

    fn diagnostics_for_stale(body: &str) -> Vec<Diagnostic> {
        diagnostics_in_lane(&make_config(), "CHE", true, body)
    }

    fn diagnostics_for_prefix(prefix: &str, body: &str) -> Vec<Diagnostic> {
        diagnostics_in_lane(&make_config(), prefix, false, body)
    }

    fn assert_rule_fires(diags: &[Diagnostic], rule: &str, body: &str) {
        assert!(
            diags.iter().any(|d| d.rule == rule),
            "expected {rule}, got: {diags:?}\nbody:\n{body}"
        );
    }

    fn assert_rule_silent(diags: &[Diagnostic], rule: &str, body: &str) {
        assert!(
            !diags.iter().any(|d| d.rule == rule),
            "expected no {rule}, got: {diags:?}\nbody:\n{body}"
        );
    }

    fn assert_fires(rule: &str, body: &str) {
        assert_rule_fires(&diagnostics_for(body), rule, body);
    }

    fn assert_silent(rule: &str, body: &str) {
        assert_rule_silent(&diagnostics_for(body), rule, body);
    }

    fn assert_fires_stale(rule: &str, body: &str) {
        assert_rule_fires(&diagnostics_for_stale(body), rule, body);
    }

    fn assert_silent_stale(rule: &str, body: &str) {
        assert_rule_silent(&diagnostics_for_stale(body), rule, body);
    }

    fn assert_silent_prefix(prefix: &str, rule: &str, body: &str) {
        assert_rule_silent(&diagnostics_for_prefix(prefix, body), rule, body);
    }

    fn assert_fires_with(rule: &str, needle: &str, body: &str) {
        let diags = diagnostics_for(body);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == rule && d.message.contains(needle)),
            "expected {rule} mentioning `{needle}`, got: {diags:?}\nbody:\n{body}"
        );
    }

    fn ordered_body(sections: &[(&str, &str)]) -> String {
        let mut out = String::from(
            "# CHE-0001. Test ADR\n\nDate: 2026-04-25\nLast-reviewed: 2026-04-25\nTier: S\nStatus: Accepted\n\n",
        );
        for (heading, section) in sections {
            let _ = write!(out, "## {heading}\n\n{section}\n\n");
        }
        out
    }

    fn legacy_only_body() -> String {
        format!(
            "# CHE-0001. Test ADR\n\nDate: 2026-04-25\nLast-reviewed: 2026-04-25\nTier: S\n\n\
             ## Status\n\nDeprecated\n\n## Related\n\nRoot: CHE-0001\n\n\
             ## Context\n\n{}\n\n## Decision\n\nR1 [5]: {}\n\n## Consequences\n\n{}\n",
            prose(15),
            prose(8),
            prose(15)
        )
    }

    fn metadata_and_legacy_body() -> String {
        format!(
            "# CHE-0001. Test ADR\n\nDate: 2026-04-25\nLast-reviewed: 2026-04-25\nTier: S\n\
             Status: Accepted\n\n## Status\n\nDeprecated\n\n## Related\n\nRoot: CHE-0001\n\n\
             ## Context\n\n{}\n\n## Decision\n\nR1 [5]: {}\n\n## Consequences\n\n{}\n",
            prose(15),
            prose(8),
            prose(15)
        )
    }

    fn empty_legacy_body() -> String {
        format!(
            "# CHE-0001. Test ADR\n\nDate: 2026-04-25\nLast-reviewed: 2026-04-25\nTier: S\n\n\
             ## Status\n\n## Related\n\nRoot: CHE-0001\n\n\
             ## Context\n\n{}\n\n## Decision\n\nR1 [5]: {}\n\n## Consequences\n\n{}\n",
            prose(15),
            prose(8),
            prose(15)
        )
    }

    fn make_config() -> Config {
        toml::from_str(
            r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[domains]]
prefix = "GND"
name = "Ground"
directory = "ground"
description = "Foundation test"
crates = []
foundation = true

[[rules]]
id = "T015"
params = { min_words = 7, max_words = 50 }

[[rules]]
id = "T016"
params = { max_rules = 10, min_rule_words = 7, max_rule_words = 60 }
"#,
        )
        .unwrap()
    }

    fn non_default_min_words_config() -> Config {
        toml::from_str(
            r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 20, max_words = 200 }
"#,
        )
        .unwrap()
    }

    fn enforced_min_words(config: &Config, tier: Tier) -> u64 {
        let body = Doc::new().tier(&tier.to_string()).context("one").body();
        let marker = format!("{tier}-tier minimum ");
        diagnostics_in_lane(config, "CHE", false, &body)
            .iter()
            .filter(|d| d.rule == "T015")
            .find_map(|d| {
                d.message
                    .split(&marker)
                    .nth(1)?
                    .split(')')
                    .next()?
                    .trim()
                    .parse::<u64>()
                    .ok()
            })
            .expect("a one-word section under a configured minimum must raise T015")
    }

    #[test]
    fn guidance_tier_scaling_cites_the_enforced_minimum() {
        let config = non_default_min_words_config();
        for tier in Tier::all() {
            let enforced = enforced_min_words(&config, *tier);
            let row = crate::guidelines::tier_scaling_row(&config, *tier);
            assert!(
                row.contains(&format!("min_words={enforced}")),
                "guidance row for {tier:?} must cite the enforced minimum \
                 {enforced} under a non-default configured base, got: {row}"
            );
        }
    }

    #[test]
    fn valid_record_produces_no_diagnostics() {
        let body = Doc::new().tier("B").body();
        let diags = diagnostics_for(&body);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn missing_tier_produces_t004() {
        assert_fires("T004", &Doc::new().no_tier().body());
    }

    #[test]
    fn invalid_tier_is_diagnosed_distinctly_from_missing_tier() {
        let missing = Doc::new().no_tier().body();
        let invalid = Doc::new().tier("Z").body();

        let missing_msg = diagnostics_for(&missing)
            .into_iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message)
            .expect("missing tier must produce T004");
        let invalid_msg = diagnostics_for(&invalid)
            .into_iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message)
            .expect("invalid tier must produce T004, not silence");

        assert!(
            invalid_msg.contains("unrecognized") && invalid_msg.contains('Z'),
            "invalid-tier diagnostic must name the offending value, got: {invalid_msg}"
        );
        assert_ne!(
            missing_msg, invalid_msg,
            "an invalid Tier value must not report as a missing one"
        );
    }

    #[test]
    fn missing_last_reviewed_all_tiers_is_warning() {
        for tier in Tier::all() {
            let body = Doc::new().tier(&tier.to_string()).no_last_reviewed().body();
            assert_fires("T003", &body);
        }
    }

    #[test]
    fn parenthetical_status_produces_t006() {
        let body = Doc::new()
            .status("Accepted (supersedes original u64 design)")
            .body();
        let diags = diagnostics_for(&body);
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T006").count(),
            1,
            "expected exactly one T006, got: {diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T006" && d.message.contains("remove annotations")),
            "parenthetical-specific remediation must survive: {diags:?}"
        );
    }

    #[test]
    fn amended_status_produces_t006() {
        let body = Doc::new().status("Amended 2026-04-25 — note").body();
        let diags = diagnostics_for(&body);
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T006").count(),
            1,
            "Amended should trigger exactly one T006 as invalid, got: {diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T006" && d.message.contains("unrecognized status")),
            "generic remediation expected: {diags:?}"
        );
    }

    #[test]
    fn empty_related_produces_t007() {
        let body = Doc::new().related("").body();
        assert_fires_with("T007", "no relationships", &body);
    }

    #[test]
    fn related_with_relationship_no_t007() {
        assert_silent("T007", &Doc::new().body());
    }

    #[test]
    fn code_block_at_limit_no_t011() {
        assert_silent("T011", &planted_body(20));
    }

    #[test]
    fn code_block_over_limit_produces_t011() {
        let body = planted_body(21);
        let fence = fence_line(&body);
        let diags = diagnostics_for(&body);
        let t011 = diags.iter().find(|d| d.rule == "T011");
        assert!(t011.is_some(), "expected T011, got: {diags:?}");
        assert_eq!(
            t011.unwrap().line,
            fence,
            "T011 should point to opening fence"
        );
    }

    #[test]
    fn section_out_of_order_produces_t014() {
        let body = ordered_body(&[
            ("Context", &prose(15)),
            ("Related", "Root: CHE-0001"),
            ("Decision", &format!("R1 [5]: {}", prose(8))),
            ("Consequences", &prose(15)),
        ]);
        assert_fires("T014", &body);
    }

    #[test]
    fn section_correct_order_no_t014() {
        assert_silent("T014", &Doc::new().body());
    }

    #[test]
    fn section_correct_order_with_legacy_status_no_t014() {
        let body = ordered_body(&[
            ("Status", "Accepted"),
            ("Related", "Root: CHE-0001"),
            ("Context", &prose(15)),
            ("Decision", &format!("R1 [5]: {}", prose(8))),
            ("Consequences", &prose(15)),
        ]);
        assert_silent("T014", &body);
    }

    #[test]
    fn section_too_few_words_produces_t015() {
        let body = Doc::new().context(&prose(3)).body();
        assert_fires_with("T015", "minimum", &body);
    }

    #[test]
    fn section_too_many_words_produces_t015() {
        let body = Doc::new().tier("B").context(&prose(60)).body();
        assert_fires_with("T015", "limit", &body);
    }

    #[test]
    fn section_within_range_no_t015() {
        assert_silent("T015", &Doc::new().body());
    }

    #[test]
    fn stale_adr_without_retirement_produces_s004() {
        let body = stale_stub().without_extra("Retirement").body();
        assert_fires_stale("S004", &body);
    }

    #[test]
    fn stale_adr_with_retirement_no_s004() {
        assert_silent_stale("S004", &stale_stub().body());
    }

    #[test]
    fn active_adr_with_retirement_produces_s005() {
        let body = Doc::new().extra("Retirement", &prose(15)).body();
        assert_fires("S005", &body);
    }

    #[test]
    fn rejected_in_active_dir_produces_s006() {
        let body = Doc::new().status("Rejected").body();
        assert_fires_with("S006", "Action:", &body);
    }

    #[test]
    fn superseded_in_active_dir_produces_s006() {
        let body = Doc::new().status("Superseded by CHE-0099").body();
        assert_fires("S006", &body);
    }

    #[test]
    fn accepted_in_active_dir_no_s006() {
        assert_silent("S006", &Doc::new().body());
    }

    #[test]
    fn tagged_rules_present_no_t016() {
        assert_silent("T016", &Doc::new().body());
    }

    #[test]
    fn no_tagged_rules_produces_t016() {
        let body = Doc::new()
            .decision("We adopt the following approach without tagging any rule here.")
            .body();
        assert_fires("T016", &body);
    }

    #[test]
    fn malformed_rule_candidate_is_diagnosed_with_line_and_raw_text() {
        let body = Doc::new()
            .decision(&format!(
                "R1 [5]: {}\nR2 [x]: Layer is not numeric so this candidate does not conform",
                prose(8)
            ))
            .body();
        let line = line_of(&body, "R2 [x]:");
        let diags = diagnostics_for(&body);
        let diag = diags
            .iter()
            .find(|d| d.rule == "T016" && d.line == line)
            .unwrap_or_else(|| {
                panic!("a malformed rule-shaped line must be diagnosed, got: {diags:?}")
            });
        assert!(
            diag.message.contains("R2 [x]:"),
            "AFM-0012:R1-R3 conformance applies to every tagged rule; the \
             diagnostic must quote the offending raw text, got: {}",
            diag.message
        );
    }

    #[test]
    fn one_valid_rule_does_not_launder_a_malformed_sibling() {
        let body = Doc::new()
            .decision(&format!(
                "R1 [5]: {}\nR3: No layer bracket at all on this candidate line",
                prose(8)
            ))
            .body();
        let line = line_of(&body, "R3:");
        assert!(
            diagnostics_for(&body)
                .iter()
                .any(|d| d.rule == "T016" && d.line == line),
            "a Decision section with one valid rule and one malformed one must \
             not pass T016 clean"
        );
    }

    #[test]
    fn malformed_rule_candidates_are_skipped_on_stale() {
        let body = Doc::new()
            .decision(&format!(
                "R1 [5]: {}\nR3: No layer bracket at all on this candidate line",
                prose(8)
            ))
            .body();
        assert_silent_stale("T016", &body);
    }

    #[test]
    fn empty_rules_produces_t016() {
        let body = Doc::new().decision("").body();
        assert_fires("T016", &body);
    }

    #[test]
    fn draft_not_exempt_from_t016() {
        let body = Doc::new()
            .status("Draft")
            .decision("We adopt the following approach without tagging any rule here.")
            .body();
        assert_fires("T016", &body);
    }

    #[test]
    fn too_many_rules_produces_t016() {
        let rules = numbered_rules(11, |_| 5);
        let body = Doc::new().tier("B").decision(&rules).body();
        assert_fires_with("T016", "limit", &body);
    }

    #[test]
    fn ten_rules_within_limit() {
        let rules = numbered_rules(10, |_| 5);
        let body = Doc::new().tier("B").decision(&rules).body();
        assert_silent("T016", &body);
    }

    #[test]
    fn rule_too_few_words_produces_t016() {
        let body = Doc::new().decision("R1 [5]: Too short").body();
        assert_fires_with("T016", "minimum", &body);
    }

    #[test]
    fn rule_too_many_words_produces_t016() {
        let body = Doc::new()
            .decision(&format!("R1 [5]: {}", prose(61)))
            .body();
        assert_fires_with("T016", "maximum", &body);
    }

    #[test]
    fn sixty_word_rule_within_limit() {
        let body = Doc::new()
            .decision(&format!("R1 [5]: {}", prose(60)))
            .body();
        assert_silent("T016", &body);
    }

    #[test]
    fn duplicate_and_gap_diagnostics_are_distinct() {
        for (ids, expected) in [
            (vec!["R1", "R1"], "Duplicate tagged rule ID R1"),
            (
                vec!["R1", "R3"],
                "Tagged rule IDs not sequential (gap after R1)",
            ),
            (
                vec!["R2"],
                "Tagged rule IDs not sequential (gap after start)",
            ),
            (vec!["R2", "R1"], ""),
            (vec!["R1", "R01"], "Duplicate tagged rule ID R1"),
        ] {
            let decision = ids
                .into_iter()
                .map(|id| format!("{id} [5]: {}", prose(8)))
                .collect::<Vec<_>>()
                .join("\n");
            let body = Doc::new().decision(&decision).body();
            let messages: Vec<_> = diagnostics_for(&body)
                .into_iter()
                .filter(|d| d.rule == "T016")
                .map(|d| d.message)
                .collect();
            let expected: Vec<_> = if expected.is_empty() {
                vec![]
            } else {
                vec![expected.to_owned()]
            };
            assert_eq!(messages, expected);
        }
    }

    #[test]
    fn numeric_layer_diagnostics_preserve_source_values() {
        for layer in [
            "0",
            "1",
            "12",
            "13",
            "255",
            "256",
            "999999999999999999999999999999999999999999",
        ] {
            let body = format!(
                "# CHE-0001. Layer\n\nTier: S\n\n## Decision\nR1 [{layer}]: This rule has enough words for the minimum word count\n"
            );
            let crate::parser::ParseFileOutcome::Parsed {
                record,
                diagnostics,
            } = crate::parser::parse_body("CHE-0001-layer.md", &body, "CHE", false)
            else {
                panic!("numeric layers must remain parseable");
            };
            assert!(diagnostics.is_empty());
            assert_eq!(record.decision_rules().len(), 1);
            assert_eq!(
                record.decision_rules()[0].layer,
                layer.parse::<u8>().unwrap_or(0)
            );
            let rule_line = line_of(&body, "R1 [");
            let mut diags = Vec::new();
            check_tagged_rules(&record, 7, 60, &mut diags);
            let expected = match layer {
                "1" | "12" => vec![],
                _ => vec![format!(
                    "Rule R1 has layer {layer} (must be 1-12, Meadows leverage points)"
                )],
            };
            assert_eq!(
                diags.iter().map(|d| d.message.clone()).collect::<Vec<_>>(),
                expected
            );
            for diagnostic in diags {
                assert_eq!((diagnostic.rule, diagnostic.line), ("T016", rule_line));
            }
        }
    }

    #[test]
    fn non_sequential_ids_produces_t016() {
        let body = Doc::new()
            .decision(&format!("R1 [5]: {}\nR3 [5]: {}", prose(8), prose(8)))
            .body();
        assert_fires_with("T016", "gap", &body);
    }

    #[test]
    fn legacy_status_section_produces_t005c() {
        let body = legacy_only_body();
        let diags = diagnostics_for(&body);
        let t005c: Vec<_> = diags.iter().filter(|d| d.rule == "T005c").collect();
        assert_eq!(
            t005c.len(),
            1,
            "legacy-only should produce EXACTLY one T005c, got: {diags:?}"
        );
        assert!(
            t005c[0].message.contains("migrate"),
            "legacy-only T005c should advise migration, got: {:?}",
            t005c[0].message
        );
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T005").count(),
            0,
            "legacy-only must not also report status missing, got: {diags:?}"
        );
    }

    #[test]
    fn legacy_section_with_metadata_field_produces_one_t005c() {
        let body = metadata_and_legacy_body();
        let heading_line = line_of(&body, "## Status");
        let diags = diagnostics_for(&body);
        let t005c: Vec<_> = diags.iter().filter(|d| d.rule == "T005c").collect();
        assert_eq!(
            t005c.len(),
            1,
            "dead leftover legacy section should produce EXACTLY one T005c, got: {diags:?}"
        );
        assert!(
            t005c[0].message.contains("delete"),
            "both-present T005c should advise deleting the leftover section, got: {:?}",
            t005c[0].message
        );
        assert_eq!(
            t005c[0].line, heading_line,
            "T005c MUST point at the legacy heading it asks the user to delete, \
             not at the preamble metadata line, got: {:?}",
            t005c[0]
        );
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T005").count(),
            0,
            "both-present must not report status missing, got: {diags:?}"
        );
    }

    #[test]
    fn metadata_status_field_no_t005c() {
        assert_silent("T005c", &Doc::new().body());
    }

    #[test]
    fn no_status_anywhere_no_t005c() {
        let body = Doc::new().no_status().body();
        assert_fires("T005", &body);
        assert_silent("T005c", &body);
    }

    #[test]
    fn empty_legacy_status_section_still_reports_t005() {
        let body = empty_legacy_body();
        let heading_line = line_of(&body, "## Status");
        let diags = diagnostics_for(&body);
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T005").count(),
            1,
            "a legacy section carrying no value is still status-less; T005 MUST \
             keep firing, got: {diags:?}"
        );
        let t005c: Vec<_> = diags.iter().filter(|d| d.rule == "T005c").collect();
        assert_eq!(
            t005c.len(),
            1,
            "an empty legacy section is still a legacy section, got: {diags:?}"
        );
        assert!(
            !t005c[0].message.contains("migrate"),
            "an empty legacy section supplied no status value, so T005c MUST NOT \
             claim the status came from it, got: {:?}",
            t005c[0].message
        );
        assert_eq!(
            t005c[0].line, heading_line,
            "T005c MUST point at the legacy heading line, got: {:?}",
            t005c[0]
        );
    }

    #[test]
    fn t015_s_tier_allows_more_words() {
        let body = Doc::new().tier("S").context(&prose(70)).body();
        assert_silent("T015", &body);
    }

    #[test]
    fn t015_d_tier_tighter_limit() {
        let body = Doc::new().tier("D").context(&prose(35)).body();
        assert_fires_with("T015", "D-tier", &body);
    }

    #[test]
    fn t015_s_tier_higher_minimum() {
        let body = Doc::new().tier("S").context(&prose(10)).body();
        assert_fires_with("T015", "S-tier minimum", &body);
    }

    #[test]
    fn t016_d_tier_fewer_rules_allowed() {
        let rules = numbered_rules(7, |_| 10);
        let body = Doc::new().tier("D").decision(&rules).body();
        assert_fires_with("T016", "D-tier", &body);
    }

    #[test]
    fn t016_layer_zero_is_warning() {
        let body = Doc::new().decision(&format!("R1 [0]: {}", prose(8))).body();
        let diags = diagnostics_for(&body);
        let layer_diag = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("layer 0"));
        assert!(
            layer_diag.is_some(),
            "layer=0 should produce T016 warning, got: {diags:?}"
        );
        assert_eq!(
            layer_diag.unwrap().severity,
            crate::report::Severity::Warning,
            "layer validation must be warning severity per AFM-0003"
        );
    }

    #[test]
    fn t016_layer_thirteen_is_warning() {
        let body = Doc::new()
            .decision(&format!("R1 [13]: {}", prose(8)))
            .body();
        let diags = diagnostics_for(&body);
        let layer_diag = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("layer 13"));
        assert!(
            layer_diag.is_some(),
            "layer=13 should produce T016 warning, got: {diags:?}"
        );
        assert_eq!(
            layer_diag.unwrap().severity,
            crate::report::Severity::Warning,
            "layer validation must be warning severity per AFM-0003"
        );
    }

    #[test]
    fn t016_layer_valid_no_error() {
        assert_silent(
            "T016",
            &Doc::new().decision(&format!("R1 [5]: {}", prose(8))).body(),
        );
    }

    #[test]
    fn t016_layer_boundary_one_and_twelve_pass() {
        let body = Doc::new()
            .tier("D")
            .decision(&format!("R1 [1]: {}\nR2 [12]: {}", prose(8), prose(8)))
            .body();
        let layer_errs: Vec<_> = diagnostics_for(&body)
            .into_iter()
            .filter(|d| d.rule == "T016" && d.message.contains("layer"))
            .collect();
        assert!(
            layer_errs.is_empty(),
            "layers 1 and 12 are valid boundaries, got: {layer_errs:?}"
        );
    }

    #[test]
    fn t019_aligned_rules_no_warning() {
        let body = Doc::new()
            .tier("B")
            .decision(&format!("R1 [5]: {}", prose(8)))
            .body();
        assert_silent("T019", &body);
    }

    #[test]
    fn t019_equal_tier_rank_passes() {
        let body = Doc::new()
            .tier("A")
            .decision(&format!("R1 [4]: {}", prose(8)))
            .body();
        assert_silent("T019", &body);
    }

    #[test]
    fn t019_lower_leverage_rule_passes() {
        let body = Doc::new()
            .tier("A")
            .decision(&format!("R1 [7]: {}", prose(8)))
            .body();
        assert_silent("T019", &body);
    }

    #[test]
    fn t019_higher_leverage_rule_fires() {
        let body = Doc::new()
            .tier("D")
            .decision(&format!("R1 [1]: {}", prose(8)))
            .body();
        assert_fires("T019", &body);
    }

    #[test]
    fn t019_adjacent_tier_higher_leverage_fires() {
        let body = Doc::new()
            .tier("B")
            .decision(&format!("R1 [4]: {}", prose(8)))
            .body();
        assert_fires("T019", &body);
    }

    #[test]
    fn t019_large_distance_produces_warning() {
        let body = Doc::new()
            .tier("D")
            .decision(&format!("R1 [1]: {}", prose(8)))
            .body();
        assert_fires_with("T019", "4 tiers", &body);
    }

    #[test]
    fn t019_rule_two_tiers_below_adr_tier_no_warning() {
        let body = Doc::new()
            .tier("S")
            .decision(&format!("R1 [5]: {}", prose(8)))
            .body();
        assert_silent("T019", &body);
    }

    #[test]
    fn t019_gnd_prefix_rule_two_tiers_below_adr_tier_no_warning() {
        let body = Doc::new()
            .prefix("GND")
            .tier("S")
            .decision(&format!("R1 [5]: {}", prose(8)))
            .body();
        assert_silent_prefix("GND", "T019", &body);
    }

    #[test]
    fn t019_c_adr_rule_one_tier_below_no_warning() {
        let body = Doc::new()
            .prefix("GND")
            .tier("C")
            .decision(&format!("R1 [9]: {}", prose(8)))
            .body();
        assert_silent_prefix("GND", "T019", &body);
    }

    #[test]
    fn t019_equal_tier_no_warning() {
        let body = Doc::new()
            .prefix("GND")
            .tier("A")
            .decision(&format!("R1 [4]: {}", prose(8)))
            .body();
        assert_silent_prefix("GND", "T019", &body);
    }

    #[test]
    fn t019_zzz_prefix_rule_two_tiers_below_adr_tier_no_warning() {
        let body = Doc::new()
            .prefix("ZZZ")
            .tier("S")
            .decision(&format!("R1 [5]: {}", prose(8)))
            .body();
        assert_silent_prefix("ZZZ", "T019", &body);
    }

    #[test]
    fn t020_within_limit_no_warning() {
        let body = Doc::new().tier("B").related(&references(7)).body();
        assert_silent("T020", &body);
    }

    #[test]
    fn t020_over_limit_produces_warning() {
        let body = Doc::new().tier("B").related(&references(8)).body();
        assert_fires("T020", &body);
    }

    #[test]
    fn t020_root_and_supersedes_not_counted() {
        let body = Doc::new()
            .tier("S")
            .related("Root: CHE-0001\nSupersedes: CHE-0099\nReferences: CHE-0002")
            .body();
        assert_silent("T020", &body);
    }

    #[test]
    fn t020_s_tier_tight_limit() {
        let body = Doc::new().tier("S").related(&references(4)).body();
        assert_fires_with("T020", "S-tier", &body);
    }

    #[test]
    fn t015_fractional_rounding_uses_round_not_floor() {
        let config: Config = toml::from_str(
            r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 7, max_words = 33 }

[[rules]]
id = "T016"
params = { max_rules = 10, min_rule_words = 7, max_rule_words = 60 }
"#,
        )
        .unwrap();
        let body = Doc::new().tier("D").context(&prose(20)).body();
        let diags = diagnostics_in_lane(&config, "CHE", false, &body);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule == "T015" && d.message.contains("limit")),
            "20 words should be within D-tier limit (33*0.6=19.8→20), got: {diags:?}"
        );
    }

    #[test]
    fn t015_fractional_rounding_boundary_plus_one_fires() {
        let config: Config = toml::from_str(
            r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 7, max_words = 33 }

[[rules]]
id = "T016"
params = { max_rules = 10, min_rule_words = 7, max_rule_words = 60 }
"#,
        )
        .unwrap();
        let body = Doc::new().tier("D").context(&prose(21)).body();
        assert!(
            diagnostics_in_lane(&config, "CHE", false, &body)
                .iter()
                .any(|d| d.rule == "T015" && d.message.contains("D-tier limit 20")),
            "21 words should trigger T015 at D-tier (limit 20)"
        );
    }

    #[test]
    fn t019_missing_tier_is_indeterminate_not_defaulted_to_b() {
        let body = Doc::new().no_tier().body();
        assert_silent("T019", &body);
    }

    fn tierless_lane_body() -> String {
        Doc::new()
            .no_tier()
            .decision(&format!("R1 [1]: {}\nR2 [13]: {}", prose(8), prose(8)))
            .body()
    }

    #[test]
    fn tierless_record_still_receives_tier_independent_t016() {
        let body = tierless_lane_body();
        let diags = diagnostics_for(&body);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T016" && d.message.contains("layer 13")),
            "AFM-0012:R3 layer-range validation is tier-independent and must \
             survive tier indeterminacy, got: {diags:?}"
        );
    }

    fn count_code(body: &str, code: &str) -> usize {
        diagnostics_for(body)
            .iter()
            .filter(|d| d.rule == code)
            .count()
    }

    fn assert_lane_b_emission_withheld(fixture: fn(&str) -> String, code: &str) {
        let under_b = count_code(&fixture("B"), code);
        assert!(
            under_b > 0,
            "boundary fixture is inert: {code} must actually emit under Tier::B, \
             otherwise its absence under indeterminacy proves nothing"
        );

        for tier in ["", "Z"] {
            let under_indeterminate = count_code(&fixture(tier), code);
            assert_eq!(
                under_indeterminate,
                under_b - 1,
                "exactly the tier-scaled {code} emission must be withheld when the \
                 tier is indeterminate; tier-independent {code} emissions must survive"
            );
        }
    }

    fn t015_boundary(tier: &str) -> String {
        Doc::new().tier_line(tier).context(&prose(51)).body()
    }

    fn t016_rule_count_boundary(tier: &str) -> String {
        let rules = numbered_rules(11, |n| if n == 11 { 13 } else { 5 });
        Doc::new().tier_line(tier).decision(&rules).body()
    }

    fn t019_boundary(tier: &str) -> String {
        Doc::new()
            .tier_line(tier)
            .decision(&format!("R1 [1]: {}", prose(8)))
            .body()
    }

    fn t020_boundary(tier: &str) -> String {
        Doc::new().tier_line(tier).related(&references(8)).body()
    }

    #[test]
    fn tierless_record_withholds_t015_word_budget_verdict() {
        assert_lane_b_emission_withheld(t015_boundary, "T015");
    }

    #[test]
    fn tierless_record_withholds_t016_rule_count_verdict() {
        assert_lane_b_emission_withheld(t016_rule_count_boundary, "T016");
    }

    #[test]
    fn tierless_record_withholds_t019_tension_verdict() {
        assert_lane_b_emission_withheld(t019_boundary, "T019");
    }

    #[test]
    fn tierless_record_withholds_t020_reference_load_verdict() {
        assert_lane_b_emission_withheld(t020_boundary, "T020");
    }

    #[test]
    fn t016_rule_count_boundary_retains_lane_a_layer_diagnostic() {
        let body = t016_rule_count_boundary("");
        let rule_line = line_of(&body, "R11 ");
        let diags = diagnostics_for(&body);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T016" && d.line == rule_line && d.message.contains("layer 13")),
            "the surviving T016 on the rule-count boundary fixture must be the \
             tier-independent AFM-0012:R3 layer diagnostic, got: {diags:?}"
        );
    }

    #[test]
    fn tierless_record_gets_exactly_one_indeterminacy_warning() {
        for tier in ["", "Z"] {
            let body = Doc::new()
                .tier_line(tier)
                .decision(&format!("R1 [1]: {}\nR2 [13]: {}", prose(8), prose(8)))
                .body();
            let diags = diagnostics_for(&body);
            let indeterminate: Vec<_> = diags
                .iter()
                .filter(|d| d.message.contains("cannot be evaluated"))
                .collect();
            assert_eq!(
                indeterminate.len(),
                1,
                "exactly one explicit indeterminacy warning expected, got: {diags:?}"
            );
            assert_eq!(indeterminate[0].rule, "T004");
        }
    }

    #[test]
    fn t004_no_longer_claims_a_fallback_to_b() {
        let body = Doc::new().tier("Z").body();
        let msg = diagnostics_for(&body)
            .into_iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message)
            .expect("invalid tier must produce T004");
        assert!(
            !msg.contains("fall back to B"),
            "T004 must not advertise a fallback that no longer happens: {msg}"
        );
    }

    fn stale_stub() -> Doc {
        Doc::new()
            .status("Superseded by CHE-0002")
            .related("Supersedes: CHE-0099")
            .no_context()
            .no_decision()
            .no_consequences()
            .extra("Retirement", &prose(15))
    }

    #[test]
    fn s007_clean_stub_produces_no_s007() {
        assert_silent_stale("S007", &stale_stub().body());
    }

    #[test]
    fn s007_multiple_supersedes_edges_no_fire() {
        let body = stale_stub()
            .related("Supersedes: CHE-0099\nSupersedes: CHE-0007")
            .body();
        assert_silent_stale("S007", &body);
    }

    #[test]
    fn s007_superseded_by_verb_fires() {
        let body = stale_stub()
            .related("Supersedes: CHE-0099\nSuperseded by: CHE-0002")
            .body();
        let count = diagnostics_for_stale(&body)
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("Superseded by"))
            .count();
        assert_eq!(
            count, 1,
            "Superseded-by in stub Related should trigger S007"
        );
    }

    #[test]
    fn s007_disallowed_section_fires() {
        let body = stale_stub().context(&prose(15)).body();
        let s007: Vec<_> = diagnostics_for_stale(&body)
            .into_iter()
            .filter(|d| d.rule == "S007" && d.message.contains("## Context"))
            .collect();
        assert_eq!(s007.len(), 1, "Context in stub should trigger one S007");
        assert!(
            s007[0].message.contains("AFM-0022"),
            "S007 message must cite AFM-0022: {}",
            s007[0].message
        );
    }

    #[test]
    fn s007_multiple_disallowed_sections_fire_per_section() {
        let body = stale_stub()
            .context(&prose(15))
            .decision(&format!("R1 [5]: {}", prose(8)))
            .consequences(&prose(15))
            .body();
        let count = diagnostics_for_stale(&body)
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("must not contain"))
            .count();
        assert_eq!(
            count, 3,
            "expected one S007 per disallowed section (Context/Decision/Consequences)"
        );
    }

    #[test]
    fn s007_non_canonical_section_name_fires() {
        let body = stale_stub().extra("Notes", "brief note").body();
        let count = diagnostics_for_stale(&body)
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("## Notes"))
            .count();
        assert_eq!(
            count, 1,
            "non-canonical `## Notes` in stub should trigger S007"
        );
    }

    #[test]
    fn s007_non_lineage_verb_fires() {
        let body = stale_stub()
            .related("Supersedes: CHE-0099\nReferences: CHE-0050")
            .body();
        let rel_line = line_of(&body, "References: CHE-0050");
        let s007_verb: Vec<_> = diagnostics_for_stale(&body)
            .into_iter()
            .filter(|d| d.rule == "S007" && d.message.contains("References"))
            .collect();
        assert_eq!(
            s007_verb.len(),
            1,
            "References in stub `## Related` should trigger one S007"
        );
        assert_eq!(
            s007_verb[0].line, rel_line,
            "diagnostic must point at the relationship's line"
        );
    }

    #[test]
    fn s007_stale_with_accepted_status_no_fire() {
        let body = stale_stub().status("Accepted").context(&prose(15)).body();
        assert_silent_stale("S007", &body);
        assert_fires_stale("S008", &body);
    }

    #[test]
    fn s007_active_with_terminal_status_no_fire() {
        let body = Doc::new().status("Superseded by CHE-0002").body();
        assert_silent("S007", &body);
    }

    #[test]
    fn t007_t008_t009_t010_t016_skipped_for_stale_stub() {
        let body = stale_stub().body();
        let diags = diagnostics_for_stale(&body);
        for rule in ["T007", "T008", "T009", "T010", "T016"] {
            assert!(
                !diags.iter().any(|d| d.rule == rule),
                "{rule} must not fire on stale stub (per AFM-0022), got: {diags:?}"
            );
        }
    }

    #[test]
    fn t007_skipped_for_stale_with_no_related_section() {
        let body = stale_stub().no_related().body();
        let diags = diagnostics_for_stale(&body);
        assert!(
            !diags.iter().any(|d| d.rule == "T007"),
            "T007 must not fire on stale stub missing `## Related` (per AFM-0022), \
             got: {diags:?}"
        );
    }

    #[test]
    fn t008_t009_t010_still_fire_on_active_missing_sections() {
        let body = Doc::new()
            .no_context()
            .no_decision()
            .no_consequences()
            .body();
        for rule in ["T008", "T009", "T010"] {
            assert_fires(rule, &body);
        }
    }

    #[test]
    fn t007_still_fires_on_active_missing_related() {
        assert_fires("T007", &Doc::new().no_related().body());
    }

    #[test]
    fn t016_still_fires_on_active_missing_tagged_rules() {
        let body = Doc::new()
            .decision("We adopt the following approach without tagging any rule here.")
            .body();
        assert_fires("T016", &body);
    }

    fn fence_line(body: &str) -> usize {
        line_of(body, "```")
    }

    fn planted_body(block_lines: usize) -> String {
        let mut body = String::new();
        body.push_str("# CHE-0001. Planted Code Block\n");
        body.push_str("\nStatus: Accepted\n");
        body.push_str("\n## Context\n\n");
        body.push_str("This ADR plants a real fenced code block for reachability.\n");
        body.push_str("\n```text\n");
        assert_eq!(
            body.lines().count(),
            PLANTED_FENCE_LINE,
            "the opening fence must sit on the line the assertions cite"
        );
        for _ in 0..block_lines {
            body.push_str("planted content line\n");
        }
        body.push_str("```\n");
        body.push_str("\n## Decision\n\nR1 [5]: planted rules keep the record parseable here.\n");
        body.push_str("\n## Consequences\n\nThe planted block is measured by the parser itself.\n");
        body
    }

    const PLANTED_FENCE_LINE: usize = 9;

    #[test]
    fn t011_reachable_from_a_planted_over_limit_code_block() {
        let body = planted_body(21);
        let fence = fence_line(&body);
        let diags = diagnostics_for(&body);
        let t011 = diags.iter().find(|d| d.rule == "T011");
        assert!(t011.is_some(), "expected T011, got: {diags:?}");
        assert_eq!(
            t011.unwrap().line,
            fence,
            "T011 must point at the planted opening fence"
        );
    }

    #[test]
    fn t011_silent_on_a_planted_at_limit_code_block() {
        assert_silent("T011", &planted_body(20));
    }

    fn t023_of(date: &str, last_reviewed: &str) -> Vec<Diagnostic> {
        diagnostics_for(&Doc::new().date(date).last_reviewed(last_reviewed).body())
            .into_iter()
            .filter(|d| d.rule == "T023")
            .collect()
    }

    #[test]
    fn t023_fires_on_a_non_date_value() {
        let diags = t023_of("banana", "2026-04-25");
        assert_eq!(diags.len(), 1, "expected exactly one T023, got: {diags:?}");
        assert!(
            diags[0].message.contains("Date:") && diags[0].message.contains("banana"),
            "T023 must name the field and quote the offending value, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn t023_fires_on_an_impossible_calendar_date() {
        assert_eq!(t023_of("2026-13-45", "2026-04-25").len(), 1);
        assert_eq!(t023_of("2026-02-30", "2026-04-25").len(), 1);
        assert_eq!(t023_of("2025-02-29", "2026-04-25").len(), 1);
    }

    #[test]
    fn t023_fires_on_a_year_outside_the_ratified_range() {
        assert_eq!(t023_of("1999-04-25", "2026-04-25").len(), 1);
        assert_eq!(t023_of("2101-04-25", "2026-04-25").len(), 1);
        assert_eq!(t023_of("0226-04-25", "2026-04-25").len(), 1);
    }

    #[test]
    fn t023_fires_on_last_reviewed_independently() {
        let diags = t023_of("2026-04-25", "never");
        assert_eq!(diags.len(), 1, "expected exactly one T023, got: {diags:?}");
        assert!(
            diags[0].message.contains("Last-reviewed:"),
            "T023 must name the offending field, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn t023_silent_on_well_formed_dates_including_leap_day() {
        for date in ["2000-01-01", "2100-12-31", "2024-02-29", "2026-04-25"] {
            assert!(
                t023_of(date, date).is_empty(),
                "{date} is a valid ratified date and must not trigger T023"
            );
        }
    }

    #[test]
    fn t023_does_not_displace_the_absence_diagnostics() {
        let body = Doc::new().no_date().no_last_reviewed().body();
        let diags = diagnostics_for(&body);
        assert!(diags.iter().any(|d| d.rule == "T002"), "got: {diags:?}");
        assert!(diags.iter().any(|d| d.rule == "T003"), "got: {diags:?}");
        assert!(!diags.iter().any(|d| d.rule == "T023"), "got: {diags:?}");
    }
}
