//! File naming rules (N001–N004).
//!
//! N001: Filename must match `PREFIX-NNNN-kebab-slug.md`
//! N002: Number in filename must match H1 title ID
//! N003: Slug must be lowercase kebab-case (a-z0-9, hyphens)
//! N004: Prefix must match a configured domain

use crate::rules::catalog;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::model::{AdrRecord, parse_adr_id_from_filename_stem};
use crate::report::Diagnostic;

static N001_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Z]{2,4}-\d{4}-[a-z0-9]+(?:-[a-z0-9]+)*\.md$").expect("valid regex")
});

static KEBAB_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("valid regex"));

pub fn check_file_name(path: &Path, domain_prefixes: &[&str], diags: &mut Vec<Diagnostic>) {
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();

    let stem = file_name.strip_suffix(".md").unwrap_or(&file_name);
    let parsed = parse_adr_id_from_filename_stem(stem);

    let Some(id) = parsed.filter(|_| N001_PATTERN.is_match(&file_name)) else {
        diags.push(catalog::N001.diagnostic(
            path,
            0,
            format!(
                "filename `{file_name}` does not match pattern \
                 `PREFIX-NNNN-kebab-slug.md`"
            ),
        ));
        return;
    };

    let slug = &stem[id.prefix().len() + 6..];

    if !KEBAB_PATTERN.is_match(slug) || !has_letter_segment(slug) {
        diags.push(catalog::N003.diagnostic(
            path,
            0,
            format!(
                "slug `{slug}` is not valid kebab-case with at least one \
                 letter segment (a-z0-9, hyphens only)"
            ),
        ));
    }

    if !domain_prefixes.contains(&id.prefix()) {
        diags.push(catalog::N004.diagnostic(
            path,
            0,
            format!(
                "prefix `{}` does not match any configured domain (known: {})",
                id.prefix(),
                domain_prefixes.join(", "),
            ),
        ));
    }
}

fn has_letter_segment(slug: &str) -> bool {
    slug.split('-')
        .any(|segment| !segment.is_empty() && segment.chars().all(|c| c.is_ascii_alphabetic()))
}

pub fn check(record: &AdrRecord, _domain_prefixes: &[&str], diags: &mut Vec<Diagnostic>) {
    let Some(file_name) = record.file_path().file_name().and_then(|f| f.to_str()) else {
        return;
    };

    let Some(stem) = file_name.strip_suffix(".md") else {
        return;
    };

    if let Some(file_id) = parse_adr_id_from_filename_stem(stem)
        && (file_id.prefix() != record.id().prefix() || file_id.number() != record.id().number())
    {
        diags.push(catalog::N002.diagnostic(
            record.file_path(),
            record.title_line(),
            format!(
                "filename ID `{file_id}` does not match H1 title ID `{}`",
                record.id()
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Related, Status, Tier};
    use std::path::PathBuf;

    const TEST_PREFIXES: &[&str] = &["COM", "CHE", "PAR", "GEN"];

    fn make_record(filename: &str, prefix: &str, num: u16) -> AdrRecord {
        let mut record = AdrRecord::test_sentinel();
        *record.id_mut() = AdrId::test_new(prefix, num);
        *record.file_path_mut() = PathBuf::from(format!("docs/adr/cherry/{filename}"));
        *record.title_mut() = Some("Test".into());
        *record.title_line_mut() = 1;
        record.set_date(Some("2026-04-25".into()));
        record.set_last_reviewed(Some("2026-04-25".into()));
        record.set_tier(Some(Tier::B));
        *record.status_mut() = Some(Status::Accepted);
        *record.status_line_mut() = 8;
        *record.status_raw_mut() = Some("Accepted".into());
        record.set_related(Related::Parsed(Vec::new()));
        *record.has_context_mut() = true;
        *record.has_decision_mut() = true;
        *record.has_consequences_mut() = true;
        record
    }

    const VALID_BODY: &str = "# CHE-0001. Valid\n\nDate: 2026-04-29\nTier: B\nStatus: Accepted\n\n## Related\n\nRoot: CHE-0001\n\n## Context\n\nProse.\n";

    fn discover(files: &[&str]) -> Vec<Diagnostic> {
        let dir = tempfile::tempdir().expect("tempdir");
        for name in files {
            std::fs::write(dir.path().join(name), VALID_BODY).expect("write adr");
        }
        let domain_dir = crate::model::DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir.path().to_owned(),
        };
        crate::parser::parse_domain(&domain_dir)
            .expect("read_dir should succeed")
            .into_parts()
            .1
    }

    fn rules_of(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule).collect()
    }

    #[test]
    fn valid_filename_no_diagnostics_end_to_end() {
        let diags = discover(&["CHE-0001-design-priority-ordering.md"]);
        assert!(diags.is_empty(), "expected no diags, got: {diags:?}");
    }

    #[test]
    fn uppercase_slug_produces_n001_end_to_end() {
        let diags = discover(&["CHE-0001-Design-Priority.md"]);
        assert!(
            diags.iter().any(|d| d.rule == "N001"),
            "expected N001 from real discovery, got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn malformed_shape_produces_n001_end_to_end() {
        let diags = discover(&["CHE-1-nope.md"]);
        assert!(
            diags.iter().any(|d| d.rule == "N001"),
            "expected N001 from real discovery, got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn all_digit_slug_produces_n003_end_to_end() {
        let diags = discover(&["CHE-0001-123.md"]);
        assert!(
            diags.iter().any(|d| d.rule == "N003"),
            "AFM-0008:R4 requires at least one letter segment; got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn n003_is_not_shadowed_by_n001() {
        let diags = discover(&["CHE-0001-123.md"]);
        assert!(
            !diags.iter().any(|d| d.rule == "N001"),
            "N001 must not shadow N003 for an all-digit slug; got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn unknown_prefix_produces_n004_end_to_end() {
        let diags = discover(&["ZZZ-0001-unregistered-prefix.md"]);
        assert!(
            diags.iter().any(|d| d.rule == "N004"),
            "expected N004 from real discovery, got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn known_prefix_no_n004_end_to_end() {
        let diags = discover(&["CHE-0001-known-prefix.md"]);
        assert!(
            !diags.iter().any(|d| d.rule == "N004"),
            "known prefix should not trigger N004, got: {:?}",
            rules_of(&diags)
        );
    }

    #[test]
    fn mismatched_number_produces_n002() {
        let record = make_record("CHE-0099-test.md", "CHE", 1);
        let mut diags = Vec::new();
        check(&record, TEST_PREFIXES, &mut diags);
        assert!(
            diags.iter().any(|d| d.rule == "N002"),
            "expected N002, got: {diags:?}"
        );
    }

    #[test]
    fn unknown_prefix_produces_n004_hand_built_record_is_not_evidence() {
        let record = make_record("ZZZ-0001-test.md", "ZZZ", 1);
        let mut diags = Vec::new();
        check(&record, TEST_PREFIXES, &mut diags);
        assert!(
            diags.is_empty(),
            "record-level check owns N002 only; naming shape is decided at discovery: {diags:?}"
        );
    }
}
