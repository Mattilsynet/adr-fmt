//! Template compliance rules (T002–T022) and structure rules (S004–S008).
//!
//! T002–T005c: preamble metadata fields (Date, Last-reviewed, Tier,
//! Status; T005c flags legacy `## Status` heading). T006 status value
//! validity. T007–T010 required sections (active ADRs only). T011
//! code-block size. T014 section ordering. T015 tier-scaled prose
//! word-count range. T016 tagged-rule validation (tier-scaled count,
//! 7–60 words each). T019 rule/ADR tier tension. T020 tier-scaled
//! `References:` load. T022 omitted-MADR-template residue section
//! names on active ADRs.
//!
//! S004/S005: stale ↔ Retirement-section presence mismatch. S006:
//! terminal-status ADR not in the stale directory. S007: stale-stub
//! structure per AFM-0022 — disallowed sections or non-lineage
//! relationship verbs in stale stubs. S008: stale-directory ADR whose
//! status is still live — the reverse direction of S006.

use std::path::Path;

use crate::config::{Config, RuleParam};
use crate::model::{AdrRecord, RelVerb, Related, Status, TierField, layer_to_tier};
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
            max_words: resolve_param(config, "T015", "max_words", DEFAULT_MAX_WORDS, diags),
            min_words: resolve_param(config, "T015", "min_words", DEFAULT_MIN_WORDS, diags),
            max_rules: resolve_param(config, "T016", "max_rules", DEFAULT_MAX_RULES, diags),
            min_rule_words: resolve_param(
                config,
                "T016",
                "min_rule_words",
                DEFAULT_MIN_RULE_WORDS,
                diags,
            ),
            max_rule_words: resolve_param(
                config,
                "T016",
                "max_rule_words",
                DEFAULT_MAX_RULE_WORDS,
                diags,
            ),
        }
    }
}

fn resolve_param(
    config: &Config,
    rule: &'static str,
    key: &str,
    default: u64,
    diags: &mut Vec<Diagnostic>,
) -> u64 {
    match config.rule_param_u64(rule, key) {
        RuleParam::Value(value) => value,
        RuleParam::Absent => default,
        RuleParam::Invalid {
            rule_id,
            key,
            reason,
        } => {
            diags.push(Diagnostic::warning(
                rule,
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
fn scale(base: u64, tier: ValidTier) -> u64 {
    (base as f64 * tier.get().factor()).round() as u64
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
            let min_words = scale(budgets.min_words, tier);
            let max_words = scale(budgets.max_words, tier);
            check_section_word_counts(record, min_words, max_words, tier, diags);
            check_rule_count(record, tier, scale(budgets.max_rules, tier), diags);
            check_rule_tier_tension(record, tier, config, diags);
            check_reference_load(record, tier, diags);
            Some(min_words)
        }
        TierScaling::Indeterminate => None,
    };

    check_stale_lifecycle(record, config, effective_min, diags);
    check_stale_stub_structure(record, diags);
}

fn check_metadata(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if record.date().is_none() {
        diags.push(Diagnostic::warning(
            "T002",
            record.file_path(),
            0,
            "missing `Date:` field".into(),
        ));
    }

    if record.last_reviewed().is_none() {
        diags.push(Diagnostic::warning(
            "T003",
            record.file_path(),
            0,
            "missing `Last-reviewed:` field (required for all tiers)".into(),
        ));
    }

    match record.tier_field() {
        TierField::Valid(_) => {}
        TierField::Absent => {
            diags.push(Diagnostic::warning(
                "T004",
                record.file_path(),
                0,
                format!("missing `Tier:` field — {TIER_SCALED_CHECKS}"),
            ));
        }
        TierField::Invalid { raw } => {
            diags.push(Diagnostic::warning(
                "T004",
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
        diags.push(Diagnostic::warning(
            "T005",
            record.file_path(),
            0,
            "no status value — add a `Status:` preamble metadata field \
             (e.g., `Status: Accepted`)"
                .into(),
        ));
    }

    if record.has_legacy_status_section() {
        let message = if record.status_from_section() {
            "status uses legacy `## Status` section — migrate to \
             `Status:` preamble metadata field (e.g., `Status: Accepted`)"
                .to_owned()
        } else {
            "legacy `## Status` section is dead content — delete the leftover \
             section; the `Status:` preamble metadata field is authoritative"
                .to_owned()
        };
        diags.push(Diagnostic::warning(
            "T005c",
            record.file_path(),
            record.status_line(),
            message,
        ));
    }
}

enum LiveStatus {
    Draft,
    Proposed,
    Accepted,
}

enum StatusLiveness {
    Live(LiveStatus),
    Terminal,
    Indeterminate,
}

impl StatusLiveness {
    fn classify(status: &Status) -> Self {
        match status {
            Status::Draft => Self::Live(LiveStatus::Draft),
            Status::Proposed => Self::Live(LiveStatus::Proposed),
            Status::Accepted => Self::Live(LiveStatus::Accepted),
            Status::Rejected | Status::Deprecated | Status::SupersededBy(_) => Self::Terminal,
            Status::Invalid(_) => Self::Indeterminate,
        }
    }
}

impl std::fmt::Display for LiveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Draft => "Draft",
            Self::Proposed => "Proposed",
            Self::Accepted => "Accepted",
        };
        f.write_str(name)
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
        diags.push(Diagnostic::warning(
            "T006",
            record.file_path(),
            record.status_line(),
            invalid.message(),
        ));
    }

    if record.is_stale() {
        return;
    }
    if matches!(record.related(), Related::Absent) {
        diags.push(Diagnostic::warning(
            "T007",
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
        diags.push(Diagnostic::warning("T007", record.file_path(), 0, message));
    }
}

fn check_structure(record: &AdrRecord, diags: &mut Vec<Diagnostic>) {
    if !record.is_stale() && !record.has_context() {
        diags.push(Diagnostic::warning(
            "T008",
            record.file_path(),
            0,
            "missing `## Context` section".into(),
        ));
    }

    if !record.is_stale() && !record.has_decision() {
        diags.push(Diagnostic::warning(
            "T009",
            record.file_path(),
            0,
            "missing `## Decision` section".into(),
        ));
    }

    if !record.is_stale() && !record.has_consequences() {
        diags.push(Diagnostic::warning(
            "T010",
            record.file_path(),
            0,
            "missing `## Consequences` section".into(),
        ));
    }

    if record.max_code_block_lines() > MAX_CODE_BLOCK_LINES {
        diags.push(Diagnostic::warning(
            "T011",
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

fn check_stale_lifecycle(
    record: &AdrRecord,
    config: &Config,
    effective_min: Option<u64>,
    diags: &mut Vec<Diagnostic>,
) {
    if record.is_stale() && !record.has_retirement() {
        diags.push(Diagnostic::warning(
            "S004",
            record.file_path(),
            0,
            "stale ADR missing `## Retirement` section — explain why \
             this ADR was retired"
                .into(),
        ));
    }

    if !record.is_stale() && record.has_retirement() {
        diags.push(Diagnostic::warning(
            "S005",
            record.file_path(),
            0,
            "active ADR has `## Retirement` section — Retirement is \
             only for stale ADRs"
                .into(),
        ));
    }

    if let Some(ref status) = record.status()
        && status.is_terminal()
        && !record.is_stale()
    {
        let status_display = match status {
            Status::Rejected => "Rejected".to_string(),
            Status::Deprecated => "Deprecated".to_string(),
            Status::SupersededBy(id) => format!("Superseded by {id}"),
            _ => format!("{status:?}"),
        };
        let retirement_budget = match effective_min {
            Some(min_words) => format!(" (≥{min_words} words)"),
            None => String::new(),
        };
        diags.push(Diagnostic::warning(
            "S006",
            record.file_path(),
            record.status_line(),
            format!(
                "{} has terminal status '{status_display}' but is not in the \
                 stale directory. Action: move this file to {stale_dir}/ and add a \
                 `## Retirement` section{retirement_budget} explaining why this \
                 ADR left active service.",
                record.id(),
                stale_dir = config.stale.directory,
            ),
        ));
    }

    if let Some(status) = record.status()
        && record.is_stale()
    {
        match StatusLiveness::classify(status) {
            StatusLiveness::Live(live) => diags.push(Diagnostic::warning(
                "S008",
                record.file_path(),
                record.status_line(),
                format!(
                    "{} is in the {stale_dir}/ directory but has non-terminal status \
                     '{live}'. Action: either set a terminal status (Rejected, \
                     Deprecated, or Superseded by PREFIX-NNNN) to record how this ADR \
                     left active service, or move the file back out of {stale_dir}/.",
                    record.id(),
                    stale_dir = config.stale.directory,
                ),
            )),
            StatusLiveness::Terminal => {}
            StatusLiveness::Indeterminate => {
                debug_assert!(
                    {
                        let file = record.file_path().display().to_string();
                        diags.iter().any(|d| d.rule == "T006" && d.file == file)
                    },
                    "an indeterminate status must already have been diagnosed by T006 \
                     before the stale lifecycle check declines to emit S008"
                );
            }
        }
    }
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
            diags.push(Diagnostic::warning(
                "S007",
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
            diags.push(Diagnostic::warning(
                "S007",
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
            diags.push(Diagnostic::warning(
                "T014",
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
            diags.push(Diagnostic::warning(
                "T022",
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
                diags.push(Diagnostic::warning(
                    "T015",
                    record.file_path(),
                    0,
                    format!(
                        "`## {section}` has {count} word(s) ({tier}-tier minimum {min_words}) — \
                         provide more context"
                    ),
                ));
            } else if (count as u64) > max_words {
                diags.push(Diagnostic::warning(
                    "T015",
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
            diags.push(Diagnostic::warning(
                "S004",
                record.file_path(),
                0,
                format!(
                    "`## Retirement` has {count} word(s) ({tier}-tier minimum {min_words}) — \
                     explain why this ADR was retired"
                ),
            ));
        } else if (count as u64) > max_words {
            diags.push(Diagnostic::warning(
                "T015",
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
        diags.push(Diagnostic::warning(
            "T016",
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

fn check_tagged_rules(
    record: &AdrRecord,
    min_rule_words: u64,
    max_rule_words: u64,
    diags: &mut Vec<Diagnostic>,
) {
    if record.is_stale() {
        return;
    }

    for candidate in record.malformed_decision_rules() {
        diags.push(Diagnostic::warning(
            "T016",
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
        diags.push(Diagnostic::warning(
            "T016",
            record.file_path(),
            0,
            "Decision section lacks tagged rules (RN [L]: pattern)".into(),
        ));
        return;
    }

    for rule in record.decision_rules() {
        let word_count = rule.text.split_whitespace().count() as u64;
        if word_count < min_rule_words {
            diags.push(Diagnostic::warning(
                "T016",
                record.file_path(),
                rule.line,
                format!(
                    "Rule {id} has {word_count} word(s) (minimum {min_rule_words})",
                    id = rule.id,
                ),
            ));
        } else if word_count > max_rule_words {
            diags.push(Diagnostic::warning(
                "T016",
                record.file_path(),
                rule.line,
                format!(
                    "Rule {id} has {word_count} word(s) (maximum {max_rule_words}) — be concise",
                    id = rule.id,
                ),
            ));
        }

        if rule.layer == 0 || rule.layer > 12 {
            diags.push(Diagnostic::warning(
                "T016",
                record.file_path(),
                rule.line,
                format!(
                    "Rule {id} has layer {layer} (must be 1-12, Meadows leverage points)",
                    id = rule.id,
                    layer = rule.layer,
                ),
            ));
        }
    }

    let mut nums: Vec<u32> = Vec::new();
    for rule in record.decision_rules() {
        if let Some(num_str) = rule.id.strip_prefix('R')
            && let Ok(num) = num_str.parse::<u32>()
        {
            nums.push(num);
        }
    }

    nums.sort_unstable();
    for (i, &num) in nums.iter().enumerate() {
        let expected = u32::try_from(i).expect("rule count fits u32") + 1;
        if num != expected {
            let prev = if i > 0 {
                format!("R{}", nums[i - 1])
            } else {
                "start".into()
            };
            diags.push(Diagnostic::warning(
                "T016",
                record.file_path(),
                0,
                format!("Tagged rule IDs not sequential (gap after {prev})"),
            ));
            return;
        }
    }
}

fn check_rule_tier_tension(
    record: &AdrRecord,
    adr_tier: ValidTier,
    config: &Config,
    diags: &mut Vec<Diagnostic>,
) {
    let _ = config;
    let adr_rank = adr_tier.get().rank();

    for rule in record.decision_rules() {
        let Some(rule_tier) = layer_to_tier(rule.layer) else {
            continue;
        };
        let rule_rank = rule_tier.rank();
        if rule_rank < adr_rank {
            let distance = adr_rank - rule_rank;
            diags.push(Diagnostic::warning(
                "T019",
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
        diags.push(Diagnostic::warning(
            "T020",
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
    use crate::model::{AdrId, Tier};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn check(record: &AdrRecord, config: &Config, diags: &mut Vec<Diagnostic>) {
        let budgets = Budgets::resolve(config, diags);
        super::check(record, config, &budgets, diags);
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

    fn make_record() -> AdrRecord {
        let mut word_counts = HashMap::new();
        word_counts.insert("Context".into(), 15);
        word_counts.insert("Decision".into(), 15);
        word_counts.insert("Consequences".into(), 15);

        let mut record = AdrRecord::test_sentinel();
        *record.id_mut() = AdrId::test_new("CHE", 1);
        *record.file_path_mut() = PathBuf::from("test.md");
        *record.title_mut() = Some("Test".into());
        *record.title_line_mut() = 1;
        *record.date_mut() = Some("2026-04-25".into());
        *record.last_reviewed_mut() = Some("2026-04-25".into());
        record.set_tier(Some(Tier::S));
        *record.status_mut() = Some(Status::Accepted);
        *record.status_line_mut() = 8;
        *record.status_raw_mut() = Some("Accepted".into());
        *record.has_context_mut() = true;
        *record.has_decision_mut() = true;
        *record.has_consequences_mut() = true;
        *record.section_order_mut() = vec![
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        *record.section_word_counts_mut() = word_counts;
        record
    }

    #[test]
    fn valid_record_produces_no_diagnostics() {
        use crate::model::{RelVerb, Relationship, TaggedRule};
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.relationships_mut() = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id().clone(),
            line: 10,
        }];
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers".into(),
            line: 10,
            layer: 5,
        }];

        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn missing_tier_produces_t004() {
        let mut record = make_record();
        record.set_tier(None);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(diags.iter().any(|d| d.rule == "T004"));
    }

    #[test]
    fn invalid_tier_is_diagnosed_distinctly_from_missing_tier() {
        let mut missing = make_record();
        missing.set_tier_field(TierField::Absent);
        let mut invalid = make_record();
        invalid.set_tier_field(TierField::Invalid { raw: "Z".into() });

        let config = make_config();
        let mut missing_diags = Vec::new();
        check(&missing, &config, &mut missing_diags);
        let mut invalid_diags = Vec::new();
        check(&invalid, &config, &mut invalid_diags);

        let missing_msg = missing_diags
            .iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message.clone())
            .expect("missing tier must produce T004");
        let invalid_msg = invalid_diags
            .iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message.clone())
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
        for tier in [Tier::S, Tier::A, Tier::B, Tier::C, Tier::D] {
            let mut record = make_record();
            record.set_tier(Some(tier));
            *record.last_reviewed_mut() = None;
            let config = make_config();
            let mut diags = Vec::new();
            check(&record, &config, &mut diags);
            assert!(
                diags.iter().any(|d| d.rule == "T003"),
                "expected T003 for tier {tier:?}"
            );
        }
    }

    #[test]
    fn parenthetical_status_produces_t006() {
        let mut record = make_record();
        *record.status_raw_mut() = Some("Accepted (supersedes original u64 design)".into());
        *record.status_mut() = Some(Status::Invalid(
            "Accepted (supersedes original u64 design)".into(),
        ));
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        let mut record = make_record();
        *record.status_raw_mut() = Some("Amended 2026-04-25 — note".into());
        *record.status_mut() = Some(Status::Invalid("Amended 2026-04-25 — note".into()));
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        let mut record = make_record();
        *record.relationships_mut() = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T007"),
            "empty Related should trigger T007, got: {diags:?}"
        );
    }

    #[test]
    fn related_with_relationship_no_t007() {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_record();
        *record.relationships_mut() = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id().clone(),
            line: 10,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T007"),
            "Related with relationship should not trigger T007"
        );
    }

    #[test]
    fn code_block_at_limit_no_t011() {
        let mut record = make_record();
        *record.max_code_block_lines_mut() = 20;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T011"),
            "20 lines should not trigger T011"
        );
    }

    #[test]
    fn code_block_over_limit_produces_t011() {
        let mut record = make_record();
        *record.max_code_block_lines_mut() = 21;
        *record.max_code_block_line_mut() = 42;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t011 = diags.iter().find(|d| d.rule == "T011");
        assert!(t011.is_some(), "expected T011, got: {diags:?}");
        assert_eq!(t011.unwrap().line, 42, "T011 should point to opening fence");
    }

    #[test]
    fn section_out_of_order_produces_t014() {
        let mut record = make_record();
        *record.section_order_mut() = vec![
            "Context".into(),
            "Related".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T014"),
            "out-of-order sections should trigger T014, got: {diags:?}"
        );
    }

    #[test]
    fn section_correct_order_no_t014() {
        let record = make_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T014"),
            "correct order should not trigger T014, got: {diags:?}"
        );
    }

    #[test]
    fn section_correct_order_with_legacy_status_no_t014() {
        let mut record = make_record();
        *record.section_order_mut() = vec![
            "Status".into(),
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T014"),
            "correct legacy order should not trigger T014, got: {diags:?}"
        );
    }

    #[test]
    fn section_too_few_words_produces_t015() {
        let mut record = make_record();
        record.section_word_counts_mut().insert("Context".into(), 3);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T015"),
            "3 words should trigger T015, got: {diags:?}"
        );
    }

    #[test]
    fn section_too_many_words_produces_t015() {
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 60);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t015 = diags
            .iter()
            .find(|d| d.rule == "T015" && d.message.contains("limit"));
        assert!(
            t015.is_some(),
            "60 words should trigger T015 max, got: {diags:?}"
        );
    }

    #[test]
    fn section_within_range_no_t015() {
        let record = make_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T015"),
            "15 words should not trigger T015, got: {diags:?}"
        );
    }

    #[test]
    fn stale_adr_without_retirement_produces_s004() {
        let mut record = make_record();
        *record.is_stale_mut() = true;
        *record.has_retirement_mut() = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "S004"),
            "stale without Retirement should trigger S004, got: {diags:?}"
        );
    }

    #[test]
    fn stale_adr_with_retirement_no_s004() {
        let mut record = make_record();
        *record.is_stale_mut() = true;
        *record.has_retirement_mut() = true;
        record
            .section_word_counts_mut()
            .insert("Retirement".into(), 15);
        *record.section_order_mut() = vec![
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
            "Retirement".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s004_existence: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S004" && d.message.contains("missing"))
            .collect();
        assert!(
            s004_existence.is_empty(),
            "stale with Retirement should not trigger S004 existence check"
        );
    }

    #[test]
    fn active_adr_with_retirement_produces_s005() {
        let mut record = make_record();
        *record.is_stale_mut() = false;
        *record.has_retirement_mut() = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "S005"),
            "active with Retirement should trigger S005, got: {diags:?}"
        );
    }

    #[test]
    fn rejected_in_active_dir_produces_s006() {
        let mut record = make_record();
        *record.status_mut() = Some(Status::Rejected);
        *record.status_raw_mut() = Some("Rejected".into());
        *record.is_stale_mut() = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s006 = diags.iter().find(|d| d.rule == "S006");
        assert!(s006.is_some(), "Rejected in active dir should trigger S006");
        assert!(
            s006.unwrap().message.contains("Action:"),
            "S006 message must contain actionable instructions"
        );
    }

    #[test]
    fn superseded_in_active_dir_produces_s006() {
        let mut record = make_record();
        *record.status_mut() = Some(Status::SupersededBy(AdrId::test_new("CHE", 99)));
        *record.status_raw_mut() = Some("Superseded by CHE-0099".into());
        *record.is_stale_mut() = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s006 = diags.iter().find(|d| d.rule == "S006");
        assert!(
            s006.is_some(),
            "Superseded in active dir should trigger S006"
        );
    }

    #[test]
    fn accepted_in_active_dir_no_s006() {
        let record = make_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S006"),
            "Accepted in active dir should NOT trigger S006"
        );
    }

    #[test]
    fn tagged_rules_present_no_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![
            TaggedRule {
                id: "R1".into(),
                text: "All events must be versioned with semantic version numbers always".into(),
                line: 10,
                layer: 5,
            },
            TaggedRule {
                id: "R2".into(),
                text: "Snapshots are created at one hundred event intervals minimum always".into(),
                line: 11,
                layer: 5,
            },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T016"),
            "tagged rules should not trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn no_tagged_rules_produces_t016() {
        let mut record = make_record();
        *record.decision_rules_mut() = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "missing tagged rules should trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn malformed_rule_candidate_is_diagnosed_with_line_and_raw_text() {
        use crate::model::MalformedRule;
        let mut record = make_record();
        *record.malformed_decision_rules_mut() = vec![MalformedRule {
            line: 42,
            raw: "R2 [x]: Layer is not numeric so this candidate does not conform".into(),
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let diag = diags
            .iter()
            .find(|d| d.rule == "T016" && d.line == 42)
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
        use crate::model::MalformedRule;
        let mut record = make_record();
        *record.malformed_decision_rules_mut() = vec![MalformedRule {
            line: 11,
            raw: "R3: No layer bracket at all on this candidate line".into(),
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016" && d.line == 11),
            "a Decision section with one valid rule and one malformed one must \
             not pass T016 clean, got: {diags:?}"
        );
    }

    #[test]
    fn malformed_rule_candidates_are_skipped_on_stale() {
        use crate::model::MalformedRule;
        let mut record = make_record();
        *record.is_stale_mut() = true;
        *record.malformed_decision_rules_mut() = vec![MalformedRule {
            line: 11,
            raw: "R3: No layer bracket at all on this candidate line".into(),
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T016"),
            "T016 is skipped on stale ADRs; the malformed-candidate check \
             inherits that exemption, got: {diags:?}"
        );
    }

    #[test]
    fn empty_rules_produces_t016() {
        let mut record = make_record();
        *record.decision_rules_mut() = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "empty rules should trigger T016, got: {diags:?}"
        );
    }

    #[test]
    fn draft_not_exempt_from_t016() {
        let mut record = make_record();
        *record.status_mut() = Some(Status::Draft);
        *record.status_raw_mut() = Some("Draft".into());
        *record.decision_rules_mut() = vec![];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "Draft should NOT be exempt from T016, got: {diags:?}"
        );
    }

    #[test]
    fn too_many_rules_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.decision_rules_mut() = (1..=11)
            .map(|i| TaggedRule {
                id: format!("R{i}"),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10 + i,
                layer: 5,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016_max = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("limit"));
        assert!(
            t016_max.is_some(),
            "11 rules should trigger T016 max (B-tier limit 10), got: {diags:?}"
        );
    }

    #[test]
    fn ten_rules_within_limit() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.decision_rules_mut() = (1..=10)
            .map(|i| TaggedRule {
                id: format!("R{i}"),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10 + i,
                layer: 5,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016_max = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("limit"));
        assert!(
            t016_max.is_none(),
            "10 rules should not trigger T016 max, got: {diags:?}"
        );
    }

    #[test]
    fn rule_too_few_words_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "Too short".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("minimum"));
        assert!(
            t016.is_some(),
            "2-word rule should trigger T016 min, got: {diags:?}"
        );
    }

    #[test]
    fn rule_too_many_words_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        let long_text = (0..61).map(|_| "word").collect::<Vec<_>>().join(" ");
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: long_text,
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("maximum"));
        assert!(
            t016.is_some(),
            "61-word rule should trigger T016 max (limit 60), got: {diags:?}"
        );
    }

    #[test]
    fn sixty_word_rule_within_limit() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        let text = (0..60).map(|_| "word").collect::<Vec<_>>().join(" ");
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text,
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("maximum"));
        assert!(
            t016.is_none(),
            "60-word rule should not trigger T016 max, got: {diags:?}"
        );
    }

    #[test]
    fn non_sequential_ids_produces_t016() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![
            TaggedRule {
                id: "R1".into(),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10,
                layer: 5,
            },
            TaggedRule {
                id: "R3".into(),
                text: "This rule also has enough words to pass the minimum check".into(),
                line: 12,
                layer: 5,
            },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("gap"));
        assert!(t016.is_some(), "gap in IDs should trigger T016");
    }

    #[test]
    fn legacy_status_section_produces_t005c() {
        let mut record = make_record();
        *record.status_from_section_mut() = true;
        *record.has_legacy_status_section_mut() = true;
        *record.section_order_mut() = vec![
            "Status".into(),
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        let mut record = make_record();
        *record.status_from_section_mut() = false;
        *record.has_legacy_status_section_mut() = true;
        *record.section_order_mut() = vec![
            "Status".into(),
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
            diags.iter().filter(|d| d.rule == "T005").count(),
            0,
            "both-present must not report status missing, got: {diags:?}"
        );
    }

    #[test]
    fn metadata_status_field_no_t005c() {
        use crate::model::{RelVerb, Relationship, TaggedRule};
        let mut record = make_record();
        *record.status_from_section_mut() = false;
        *record.relationships_mut() = vec![Relationship {
            verb: RelVerb::Root,
            target: record.id().clone(),
            line: 10,
        }];
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T005c"),
            "metadata field format should not produce T005c, got: {diags:?}"
        );
    }

    #[test]
    fn no_status_anywhere_no_t005c() {
        let mut record = make_record();
        *record.status_mut() = None;
        *record.status_raw_mut() = None;
        *record.status_from_section_mut() = false;
        *record.has_legacy_status_section_mut() = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T005"),
            "missing status should produce T005, got: {diags:?}"
        );
        assert!(
            !diags.iter().any(|d| d.rule == "T005c"),
            "missing status should not produce T005c, got: {diags:?}"
        );
    }

    #[test]
    fn empty_legacy_status_section_still_reports_t005() {
        let mut record = make_record();
        *record.status_mut() = None;
        *record.status_raw_mut() = None;
        *record.status_from_section_mut() = false;
        *record.has_legacy_status_section_mut() = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T005").count(),
            1,
            "a legacy section carrying no value is still status-less; T005 MUST \
             keep firing, got: {diags:?}"
        );
        assert_eq!(
            diags.iter().filter(|d| d.rule == "T005c").count(),
            1,
            "an empty legacy section is still a legacy section, got: {diags:?}"
        );
    }

    #[test]
    fn t015_s_tier_allows_more_words() {
        let mut record = make_record();
        record.set_tier(Some(Tier::S));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 70);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T015"),
            "70 words should be within S-tier limit (75), got: {diags:?}"
        );
    }

    #[test]
    fn t015_d_tier_tighter_limit() {
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 35);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t015 = diags
            .iter()
            .find(|d| d.rule == "T015" && d.message.contains("D-tier"));
        assert!(
            t015.is_some(),
            "35 words should trigger T015 at D-tier (limit 30), got: {diags:?}"
        );
    }

    #[test]
    fn t015_s_tier_higher_minimum() {
        let mut record = make_record();
        record.set_tier(Some(Tier::S));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 10);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t015 = diags
            .iter()
            .find(|d| d.rule == "T015" && d.message.contains("S-tier minimum"));
        assert!(
            t015.is_some(),
            "10 words should trigger T015 min at S-tier (min 15), got: {diags:?}"
        );
    }

    #[test]
    fn t016_d_tier_fewer_rules_allowed() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        *record.decision_rules_mut() = (1..=7)
            .map(|i| TaggedRule {
                id: format!("R{i}"),
                text: "This rule has enough words to pass the minimum check here".into(),
                line: 10 + i,
                layer: 10,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t016 = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("D-tier"));
        assert!(
            t016.is_some(),
            "7 rules should trigger T016 at D-tier (limit 6), got: {diags:?}"
        );
    }

    #[test]
    fn t016_layer_zero_is_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 0,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 13,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
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
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let layer_err = diags
            .iter()
            .find(|d| d.rule == "T016" && d.message.contains("layer"));
        assert!(
            layer_err.is_none(),
            "layer=5 should not produce layer error, got: {diags:?}"
        );
    }

    #[test]
    fn t016_layer_boundary_one_and_twelve_pass() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        *record.decision_rules_mut() = vec![
            TaggedRule {
                id: "R1".into(),
                text: "All events must be versioned with semantic version numbers always".into(),
                line: 10,
                layer: 1,
            },
            TaggedRule {
                id: "R2".into(),
                text: "All events must be versioned with semantic version numbers always".into(),
                line: 11,
                layer: 12,
            },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let layer_errs: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "T016" && d.message.contains("layer"))
            .collect();
        assert!(
            layer_errs.is_empty(),
            "layers 1 and 12 are valid boundaries, got: {layer_errs:?}"
        );
    }

    #[test]
    fn t019_aligned_rules_no_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T019"),
            "aligned rules should not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_equal_tier_rank_passes() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::A));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 4,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T019"),
            "equal-tier must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_lower_leverage_rule_passes() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::A));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 7,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T019"),
            "lower-leverage rule must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_higher_leverage_rule_fires() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 1,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T019"),
            "higher-leverage rule must trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_adjacent_tier_higher_leverage_fires() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 4,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T019"),
            "rule at higher leverage than ADR tier must trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_large_distance_produces_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 1,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t019 = diags.iter().find(|d| d.rule == "T019");
        assert!(
            t019.is_some(),
            "distance 4 should trigger T019, got: {diags:?}"
        );
        assert!(
            t019.unwrap().message.contains("4 tiers"),
            "message should mention distance: {}",
            t019.unwrap().message
        );
    }

    #[test]
    fn t019_distance_two_lower_leverage_no_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(Some(Tier::S));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T019"),
            "S-tier ADR with lower-leverage rule must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_foundation_s_tier_lower_leverage_no_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.id_mut() = AdrId::test_new("GND", 1);
        record.set_tier(Some(Tier::S));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t019 = diags.iter().find(|d| d.rule == "T019");
        assert!(
            t019.is_none(),
            "foundation S-tier at lower leverage must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_s_tier_rule_in_c_tier_foundation_adr_lower_leverage_no_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.id_mut() = AdrId::test_new("GND", 1);
        record.set_tier(Some(Tier::C));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 9,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t019 = diags.iter().find(|d| d.rule == "T019");
        assert!(
            t019.is_none(),
            "C-tier foundation ADR with lower-leverage rule must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_equal_tier_no_warning() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.id_mut() = AdrId::test_new("GND", 1);
        record.set_tier(Some(Tier::A));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 4,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t019 = diags.iter().find(|d| d.rule == "T019");
        assert!(
            t019.is_none(),
            "equal-tier rule must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t019_unknown_prefix_no_carve_out_needed() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.id_mut() = AdrId::test_new("ZZZ", 1);
        record.set_tier(Some(Tier::S));
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 5,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t019 = diags.iter().find(|d| d.rule == "T019");
        assert!(
            t019.is_none(),
            "S-tier ADR with lower-leverage rule must not trigger T019, got: {diags:?}"
        );
    }

    #[test]
    fn t020_within_limit_no_warning() {
        use crate::model::{AdrId, RelVerb, Relationship};
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.relationships_mut() = (1..=7)
            .map(|i| Relationship {
                verb: RelVerb::References,
                target: AdrId::test_new("CHE", i),
                line: 10 + i as usize,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T020"),
            "7 refs at B-tier (limit 7) should not trigger T020, got: {diags:?}"
        );
    }

    #[test]
    fn t020_over_limit_produces_warning() {
        use crate::model::{AdrId, RelVerb, Relationship};
        let mut record = make_record();
        record.set_tier(Some(Tier::B));
        *record.relationships_mut() = (1..=8)
            .map(|i| Relationship {
                verb: RelVerb::References,
                target: AdrId::test_new("CHE", i),
                line: 10 + i as usize,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t020 = diags.iter().find(|d| d.rule == "T020");
        assert!(
            t020.is_some(),
            "8 refs at B-tier (limit 7) should trigger T020, got: {diags:?}"
        );
    }

    #[test]
    fn t020_root_and_supersedes_not_counted() {
        use crate::model::{AdrId, RelVerb, Relationship};
        let mut record = make_record();
        record.set_tier(Some(Tier::S));
        *record.relationships_mut() = vec![
            Relationship {
                verb: RelVerb::Root,
                target: record.id().clone(),
                line: 10,
            },
            Relationship {
                verb: RelVerb::Supersedes,
                target: AdrId::test_new("CHE", 99),
                line: 11,
            },
            Relationship {
                verb: RelVerb::References,
                target: AdrId::test_new("CHE", 2),
                line: 12,
            },
        ];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T020"),
            "only 1 References: should not trigger T020 at S-tier (limit 3), got: {diags:?}"
        );
    }

    #[test]
    fn t020_s_tier_tight_limit() {
        use crate::model::{AdrId, RelVerb, Relationship};
        let mut record = make_record();
        record.set_tier(Some(Tier::S));
        *record.relationships_mut() = (1..=4)
            .map(|i| Relationship {
                verb: RelVerb::References,
                target: AdrId::test_new("COM", i),
                line: 10 + i as usize,
            })
            .collect();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let t020 = diags.iter().find(|d| d.rule == "T020");
        assert!(
            t020.is_some(),
            "4 refs at S-tier (limit 3) should trigger T020, got: {diags:?}"
        );
        assert!(
            t020.unwrap().message.contains("S-tier"),
            "message should mention tier"
        );
    }

    #[test]
    fn t015_fractional_rounding_uses_round_not_floor() {
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 20);
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
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags
                .iter()
                .any(|d| d.rule == "T015" && d.message.contains("limit")),
            "20 words should be within D-tier limit (33*0.6=19.8→20), got: {diags:?}"
        );
    }

    #[test]
    fn t015_fractional_rounding_boundary_plus_one_fires() {
        let mut record = make_record();
        record.set_tier(Some(Tier::D));
        record
            .section_word_counts_mut()
            .insert("Context".into(), 21);
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
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T015" && d.message.contains("D-tier limit 20")),
            "21 words should trigger T015 at D-tier (limit 20), got: {diags:?}"
        );
    }

    #[test]
    fn t019_missing_tier_is_indeterminate_not_defaulted_to_b() {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(None);
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 1,
        }];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T019"),
            "a tier-less ADR has no tier to be in tension with; T019 must not \
             report a verdict computed from a fabricated tier, got: {diags:?}"
        );
    }

    fn tierless_lane_record() -> AdrRecord {
        use crate::model::TaggedRule;
        let mut record = make_record();
        record.set_tier(None);
        *record.decision_rules_mut() = vec![
            TaggedRule {
                id: "R1".into(),
                text: "All events must be versioned with semantic version numbers always".into(),
                line: 10,
                layer: 1,
            },
            TaggedRule {
                id: "R2".into(),
                text: "All events must carry a monotonic sequence number for ordering".into(),
                line: 11,
                layer: 13,
            },
        ];
        record
    }

    #[test]
    fn tierless_record_still_receives_tier_independent_t016() {
        let record = tierless_lane_record();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T016" && d.message.contains("layer 13")),
            "AFM-0012:R3 layer-range validation is tier-independent and must \
             survive tier indeterminacy, got: {diags:?}"
        );
    }

    fn count_code(record: &AdrRecord, code: &str) -> usize {
        let config = make_config();
        let mut diags = Vec::new();
        check(record, &config, &mut diags);
        diags.iter().filter(|d| d.rule == code).count()
    }

    fn assert_lane_b_emission_withheld(fixture: fn() -> AdrRecord, code: &str) {
        let mut scaled = fixture();
        scaled.set_tier_field(TierField::Valid(Tier::B));
        let under_b = count_code(&scaled, code);
        assert!(
            under_b > 0,
            "boundary fixture is inert: {code} must actually emit under Tier::B, \
             otherwise its absence under indeterminacy proves nothing"
        );

        for field in [TierField::Absent, TierField::Invalid { raw: "Z".into() }] {
            let mut indeterminate = fixture();
            indeterminate.set_tier_field(field);
            let under_indeterminate = count_code(&indeterminate, code);
            assert_eq!(
                under_indeterminate,
                under_b - 1,
                "exactly the tier-scaled {code} emission must be withheld when the \
                 tier is indeterminate; tier-independent {code} emissions must survive"
            );
        }
    }

    fn t015_boundary_record() -> AdrRecord {
        let mut record = make_record();
        record
            .section_word_counts_mut()
            .insert("Context".into(), 51);
        record
    }

    fn t016_rule_count_boundary_record() -> AdrRecord {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = (1..=11)
            .map(|n| TaggedRule {
                id: format!("R{n}"),
                text: "All events must carry a monotonic sequence number for ordering".into(),
                line: 9 + n,
                layer: if n == 11 { 13 } else { 5 },
            })
            .collect();
        record
    }

    fn t019_boundary_record() -> AdrRecord {
        use crate::model::TaggedRule;
        let mut record = make_record();
        *record.decision_rules_mut() = vec![TaggedRule {
            id: "R1".into(),
            text: "All events must be versioned with semantic version numbers always".into(),
            line: 10,
            layer: 1,
        }];
        record
    }

    fn t020_boundary_record() -> AdrRecord {
        use crate::model::{RelVerb, Relationship};
        let mut record = t019_boundary_record();
        *record.relationships_mut() = (0..8)
            .map(|i| Relationship {
                verb: RelVerb::References,
                target: record.id().clone(),
                line: 20 + i,
            })
            .collect();
        record
    }

    #[test]
    fn tierless_record_withholds_t015_word_budget_verdict() {
        assert_lane_b_emission_withheld(t015_boundary_record, "T015");
    }

    #[test]
    fn tierless_record_withholds_t016_rule_count_verdict() {
        assert_lane_b_emission_withheld(t016_rule_count_boundary_record, "T016");
    }

    #[test]
    fn tierless_record_withholds_t019_tension_verdict() {
        assert_lane_b_emission_withheld(t019_boundary_record, "T019");
    }

    #[test]
    fn tierless_record_withholds_t020_reference_load_verdict() {
        assert_lane_b_emission_withheld(t020_boundary_record, "T020");
    }

    #[test]
    fn t016_rule_count_boundary_retains_lane_a_layer_diagnostic() {
        let mut record = t016_rule_count_boundary_record();
        record.set_tier_field(TierField::Absent);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.rule == "T016" && d.line == 20 && d.message.contains("layer 13")),
            "the surviving T016 on the rule-count boundary fixture must be the \
             tier-independent AFM-0012:R3 layer diagnostic, got: {diags:?}"
        );
    }

    #[test]
    fn tierless_record_gets_exactly_one_indeterminacy_warning() {
        for field in [TierField::Absent, TierField::Invalid { raw: "Z".into() }] {
            let mut record = tierless_lane_record();
            record.set_tier_field(field);
            let config = make_config();
            let mut diags = Vec::new();
            check(&record, &config, &mut diags);
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
        let mut record = make_record();
        record.set_tier_field(TierField::Invalid { raw: "Z".into() });
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let msg = diags
            .iter()
            .find(|d| d.rule == "T004")
            .map(|d| d.message.clone())
            .expect("invalid tier must produce T004");
        assert!(
            !msg.contains("fall back to B"),
            "T004 must not advertise a fallback that no longer happens: {msg}"
        );
    }

    fn make_stale_stub() -> AdrRecord {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_record();
        *record.is_stale_mut() = true;
        *record.has_retirement_mut() = true;
        *record.has_context_mut() = false;
        *record.has_decision_mut() = false;
        *record.has_consequences_mut() = false;
        *record.section_order_mut() = vec!["Related".into(), "Retirement".into()];
        record.section_word_counts_mut().clear();
        record
            .section_word_counts_mut()
            .insert("Retirement".into(), 30);
        *record.status_mut() = Some(Status::SupersededBy(AdrId::test_new("CHE", 2)));
        *record.status_raw_mut() = Some("Superseded by CHE-0002".into());
        *record.relationships_mut() = vec![Relationship {
            verb: RelVerb::Supersedes,
            target: AdrId::test_new("CHE", 99),
            line: 10,
        }];
        record
    }

    #[test]
    fn s007_clean_stub_produces_no_s007() {
        let record = make_stale_stub();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S007"),
            "clean stub should not trigger S007, got: {diags:?}"
        );
    }

    #[test]
    fn s007_multiple_supersedes_edges_no_fire() {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_stale_stub();
        record.relationships_mut().push(Relationship {
            verb: RelVerb::Supersedes,
            target: AdrId::test_new("CHE", 7),
            line: 11,
        });
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S007"),
            "multiple Supersedes edges are permitted, got: {diags:?}"
        );
    }

    #[test]
    fn s007_superseded_by_verb_fires() {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_stale_stub();
        record.relationships_mut().push(Relationship {
            verb: RelVerb::SupersededBy,
            target: AdrId::test_new("CHE", 2),
            line: 12,
        });
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s007_verb: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("Superseded by"))
            .collect();
        assert_eq!(
            s007_verb.len(),
            1,
            "Superseded-by in stub Related should trigger S007, got: {diags:?}"
        );
    }

    #[test]
    fn s007_disallowed_section_fires() {
        let mut record = make_stale_stub();
        *record.section_order_mut() = vec!["Related".into(), "Context".into(), "Retirement".into()];
        *record.has_context_mut() = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s007: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("## Context"))
            .collect();
        assert_eq!(
            s007.len(),
            1,
            "Context in stub should trigger one S007, got: {diags:?}"
        );
        assert!(
            s007[0].message.contains("AFM-0022"),
            "S007 message must cite AFM-0022: {}",
            s007[0].message
        );
    }

    #[test]
    fn s007_multiple_disallowed_sections_fire_per_section() {
        let mut record = make_stale_stub();
        *record.section_order_mut() = vec![
            "Related".into(),
            "Context".into(),
            "Decision".into(),
            "Consequences".into(),
            "Retirement".into(),
        ];
        *record.has_context_mut() = true;
        *record.has_decision_mut() = true;
        *record.has_consequences_mut() = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s007_section: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("must not contain"))
            .collect();
        assert_eq!(
            s007_section.len(),
            3,
            "expected one S007 per disallowed section (Context/Decision/Consequences), got: {diags:?}"
        );
    }

    #[test]
    fn s007_non_canonical_section_name_fires() {
        let mut record = make_stale_stub();
        *record.section_order_mut() = vec!["Related".into(), "Notes".into(), "Retirement".into()];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s007: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("## Notes"))
            .collect();
        assert_eq!(
            s007.len(),
            1,
            "non-canonical `## Notes` in stub should trigger S007, got: {diags:?}"
        );
    }

    #[test]
    fn s007_non_lineage_verb_fires() {
        use crate::model::{RelVerb, Relationship};
        let mut record = make_stale_stub();
        record.relationships_mut().push(Relationship {
            verb: RelVerb::References,
            target: AdrId::test_new("CHE", 50),
            line: 12,
        });
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        let s007_verb: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "S007" && d.message.contains("References"))
            .collect();
        assert_eq!(
            s007_verb.len(),
            1,
            "References in stub `## Related` should trigger one S007, got: {diags:?}"
        );
        assert_eq!(
            s007_verb[0].line, 12,
            "diagnostic must point at the relationship's line"
        );
    }

    #[test]
    fn s007_stale_with_accepted_status_no_fire() {
        let mut record = make_stale_stub();
        *record.status_mut() = Some(Status::Accepted);
        *record.status_raw_mut() = Some("Accepted".into());
        *record.section_order_mut() = vec!["Related".into(), "Context".into(), "Retirement".into()];
        *record.has_context_mut() = true;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S007"),
            "stale + non-terminal status must not trigger S007, got: {diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.rule == "S008"),
            "S008 is the rule that covers stale + non-terminal status, got: {diags:?}"
        );
    }

    #[test]
    fn s007_active_with_terminal_status_no_fire() {
        let mut record = make_record();
        *record.is_stale_mut() = false;
        *record.status_mut() = Some(Status::SupersededBy(AdrId::test_new("CHE", 2)));
        *record.status_raw_mut() = Some("Superseded by CHE-0002".into());
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "S007"),
            "active dir + terminal status must not trigger S007 (S006 covers it), got: {diags:?}"
        );
    }

    #[test]
    fn t007_t008_t009_t010_t016_skipped_for_stale_stub() {
        let record = make_stale_stub();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        for rule in ["T007", "T008", "T009", "T010", "T016"] {
            assert!(
                !diags.iter().any(|d| d.rule == rule),
                "{rule} must not fire on stale stub (per AFM-0022), got: {diags:?}"
            );
        }
    }

    #[test]
    fn t007_skipped_for_stale_with_no_related_section() {
        let mut record = make_stale_stub();
        record.set_related(Related::Absent);
        *record.section_order_mut() = vec!["Retirement".into()];
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            !diags.iter().any(|d| d.rule == "T007"),
            "T007 must not fire on stale stub missing `## Related` (per AFM-0022), got: {diags:?}"
        );
    }

    #[test]
    fn t008_t009_t010_still_fire_on_active_missing_sections() {
        let mut record = make_record();
        *record.is_stale_mut() = false;
        *record.has_context_mut() = false;
        *record.has_decision_mut() = false;
        *record.has_consequences_mut() = false;
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        for rule in ["T008", "T009", "T010"] {
            assert!(
                diags.iter().any(|d| d.rule == rule),
                "{rule} must still fire on active ADR missing the section, got: {diags:?}"
            );
        }
    }

    #[test]
    fn t007_still_fires_on_active_missing_related() {
        let mut record = make_record();
        *record.is_stale_mut() = false;
        record.set_related(Related::Absent);
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T007"),
            "T007 must still fire on active ADR missing Related, got: {diags:?}"
        );
    }

    #[test]
    fn t016_still_fires_on_active_missing_tagged_rules() {
        let mut record = make_record();
        *record.is_stale_mut() = false;
        record.decision_rules_mut().clear();
        let config = make_config();
        let mut diags = Vec::new();
        check(&record, &config, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "T016"),
            "T016 must still fire on active ADR missing tagged rules, got: {diags:?}"
        );
    }
}
