use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{AdrId, AdrRecord};
use crate::parser::{FileParseFailure, ParseOutcome};
use crate::report::Diagnostic;

#[derive(Debug, Default)]
pub struct ScannedCorpus {
    records: Vec<AdrRecord>,
    diagnostics: Vec<Diagnostic>,
    failures: Vec<FileParseFailure>,
}

impl ScannedCorpus {
    pub fn absorb(&mut self, outcome: ParseOutcome) {
        let (records, diagnostics, failures) = outcome.into_parts();
        self.records.extend(records);
        self.diagnostics.extend(diagnostics);
        self.failures.extend(failures);
    }

    #[must_use]
    pub fn records(&self) -> &[AdrRecord] {
        &self.records
    }

    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    #[cfg(test)]
    pub(crate) fn test_of(outcome: ParseOutcome) -> Self {
        let mut scan = Self::default();
        scan.absorb(outcome);
        scan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateId {
    pub id: AdrId,
    pub paths: [PathBuf; 2],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsedTarget {
    pub path: String,
    pub rule: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum Resolution<'a> {
    Resolved(&'a AdrRecord),
    Absent,
    Indeterminate(&'a UnparsedTarget),
}

#[derive(Debug)]
pub struct CorpusIndex<'a> {
    records: &'a [AdrRecord],
    by_id: HashMap<&'a AdrId, &'a AdrRecord>,
    unparsed: HashMap<AdrId, UnparsedTarget>,
}

impl<'a> CorpusIndex<'a> {
    /// Builds the index, rejecting a corpus that assigns one `AdrId` to more
    /// than one file.
    ///
    /// Records are considered in `(prefix, number, file_path)` order, so the
    /// reported duplicate is deterministic and independent of the order in
    /// which files were scanned: when several distinct ids are duplicated,
    /// the lowest such id in that ordering is the one reported.
    ///
    /// # Errors
    ///
    /// Returns [`DuplicateId`] for the first duplicated id in the ordering
    /// above, carrying that id and the two colliding paths sorted
    /// lexicographically. Later duplicates are not reported.
    pub fn build(scan: &'a ScannedCorpus) -> Result<Self, DuplicateId> {
        let records: &'a [AdrRecord] = &scan.records;
        let mut ordered: Vec<&'a AdrRecord> = records.iter().collect();
        ordered.sort_by(|a, b| {
            a.id()
                .prefix()
                .cmp(b.id().prefix())
                .then(a.id().number().cmp(&b.id().number()))
                .then(a.file_path().cmp(b.file_path()))
        });

        let mut by_id: HashMap<&'a AdrId, &'a AdrRecord> = HashMap::with_capacity(ordered.len());
        for record in ordered {
            if let Some(existing) = by_id.insert(record.id(), record) {
                let mut paths = [
                    existing.file_path().to_path_buf(),
                    record.file_path().to_path_buf(),
                ];
                paths.sort();
                return Err(DuplicateId {
                    id: record.id().clone(),
                    paths,
                });
            }
        }

        let unparsed = collect_unparsed(&scan.failures, &by_id);

        Ok(Self {
            records,
            by_id,
            unparsed,
        })
    }

    #[must_use]
    pub fn records(&self) -> &'a [AdrRecord] {
        self.records
    }

    #[must_use]
    pub fn resolve(&self, id: &AdrId) -> Resolution<'_> {
        if let Some(record) = self.by_id.get(id) {
            return Resolution::Resolved(record);
        }
        match self.unparsed.get(id) {
            Some(unparsed) => Resolution::Indeterminate(unparsed),
            None => Resolution::Absent,
        }
    }

    #[must_use]
    pub fn get(&self, id: &AdrId) -> Option<&'a AdrRecord> {
        self.by_id.get(id).copied()
    }

    #[must_use]
    pub fn contains_key(&self, id: &AdrId) -> bool {
        self.by_id.contains_key(id)
    }
}

fn collect_unparsed(
    parse_failures: &[FileParseFailure],
    by_id: &HashMap<&AdrId, &AdrRecord>,
) -> HashMap<AdrId, UnparsedTarget> {
    let mut unparsed: HashMap<AdrId, UnparsedTarget> = HashMap::new();

    for failure in parse_failures {
        if by_id.contains_key(failure.id()) {
            continue;
        }
        unparsed
            .entry(failure.id().clone())
            .or_insert_with(|| UnparsedTarget {
                path: failure.path().display().to_string(),
                rule: failure.rule(),
            });
    }

    unparsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AdrId, Related, Status, Tier};
    use std::path::PathBuf;

    fn make_id(prefix: &str, num: u16) -> AdrId {
        AdrId::test_new(prefix, num)
    }

    fn make_record(prefix: &str, num: u16, path: &str) -> AdrRecord {
        let id = make_id(prefix, num);
        let mut record = AdrRecord::test_sentinel();
        *record.id_mut() = id;
        *record.file_path_mut() = PathBuf::from(path);
        *record.title_mut() = Some("Test".into());
        *record.title_line_mut() = 1;
        *record.date_mut() = Some("2026-04-25".into());
        *record.last_reviewed_mut() = Some("2026-04-25".into());
        record.set_tier(Some(Tier::B));
        *record.status_mut() = Some(Status::Accepted);
        *record.status_line_mut() = 8;
        *record.status_raw_mut() = Some("Accepted".into());
        record.set_related(Related::Parsed(vec![]));
        *record.has_context_mut() = true;
        *record.has_decision_mut() = true;
        *record.has_consequences_mut() = true;
        record
    }

    #[test]
    fn build_succeeds_for_unique_ids() {
        let records = vec![
            make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md"),
            make_record("CHE", 2, "docs/adr/cherry/CHE-0002-b.md"),
        ];
        let scan = ScannedCorpus::test_of(ParseOutcome::test_new(records, Vec::new()));
        let index = CorpusIndex::build(&scan).expect("unique ids must build");
        assert!(index.contains_key(&make_id("CHE", 1)));
        assert!(index.contains_key(&make_id("CHE", 2)));
        assert!(!index.contains_key(&make_id("CHE", 99)));
    }

    #[test]
    fn build_detects_duplicate_within_one_directory() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md");

        let err = build_err(vec![a, b]);
        assert_eq!(err.id, make_id("CHE", 1));
        assert_eq!(
            err.paths,
            [
                PathBuf::from("docs/adr/cherry/CHE-0001-a.md"),
                PathBuf::from("docs/adr/cherry/CHE-0001-b.md"),
            ]
        );
    }

    #[test]
    fn build_is_scan_order_independent() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md");

        let forward = build_err(vec![a.clone(), b.clone()]);
        let reversed = build_err(vec![b, a]);

        assert_eq!(
            forward, reversed,
            "duplicate diagnostic must be identical regardless of scan order"
        );
    }

    #[test]
    fn build_reports_the_lowest_duplicate_id_when_several_ids_are_duplicated() {
        let records = vec![
            make_record("CHE", 2, "docs/adr/cherry/CHE-0002-a.md"),
            make_record("CHE", 2, "docs/adr/cherry/CHE-0002-b.md"),
            make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md"),
            make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md"),
        ];
        let forward = build_err(records.clone());
        let reversed = build_err(records.into_iter().rev().collect());

        assert_eq!(
            forward.id,
            make_id("CHE", 1),
            "the lowest duplicated id must be the reported one"
        );
        assert_eq!(forward, reversed, "report must not depend on scan order");
    }

    #[test]
    fn build_orders_duplicate_candidates_by_prefix_before_number() {
        let records = vec![
            make_record("ZED", 1, "docs/adr/zed/ZED-0001-a.md"),
            make_record("ZED", 1, "docs/adr/zed/ZED-0001-b.md"),
            make_record("ACE", 9, "docs/adr/ace/ACE-0009-a.md"),
            make_record("ACE", 9, "docs/adr/ace/ACE-0009-b.md"),
        ];
        let forward = build_err(records.clone());
        let reversed = build_err(records.into_iter().rev().collect());

        assert_eq!(
            forward.id,
            make_id("ACE", 9),
            "prefix orders ahead of number when selecting the reported duplicate"
        );
        assert_eq!(forward, reversed, "report must not depend on scan order");
    }

    #[test]
    fn build_detects_duplicate_across_two_directories() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/common/CHE-0001-b.md");

        let err = build_err(vec![a, b]);
        assert_eq!(err.id, make_id("CHE", 1));
    }

    fn scanned(records: Vec<AdrRecord>, parse_failures: Vec<FileParseFailure>) -> ScannedCorpus {
        ScannedCorpus::test_of(ParseOutcome::test_new(records, parse_failures))
    }

    fn build_err(records: Vec<AdrRecord>) -> DuplicateId {
        let scan = ScannedCorpus::test_of(ParseOutcome::test_new(records, Vec::new()));
        CorpusIndex::build(&scan).expect_err("duplicate id must be rejected")
    }

    fn parse_failure(cause: crate::parser::ParseFailureCause, path: &str) -> FileParseFailure {
        let id = std::path::Path::new(path)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .and_then(crate::model::parse_adr_id_from_filename_stem)
            .expect("test fixture path must claim an ADR id");
        FileParseFailure::test_new(id, PathBuf::from(path), cause)
    }

    #[test]
    fn unparsed_target_resolves_indeterminate_not_absent() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure(
            crate::parser::ParseFailureCause::TitleMissing,
            "docs/adr/cherry/CHE-0002-broken-h1.md",
        )];
        let scan = scanned(records, diags);
        let index = CorpusIndex::build(&scan).expect("unique ids must build");

        match index.resolve(&make_id("CHE", 2)) {
            Resolution::Indeterminate(unparsed) => {
                assert_eq!(unparsed.rule, "P002");
                assert_eq!(unparsed.path, "docs/adr/cherry/CHE-0002-broken-h1.md");
            }
            other => panic!("expected Indeterminate, got {other:?}"),
        }
    }

    #[test]
    fn genuinely_absent_id_resolves_absent() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure(
            crate::parser::ParseFailureCause::TitleMissing,
            "docs/adr/cherry/CHE-0002-broken-h1.md",
        )];
        let scan = scanned(records, diags);
        let index = CorpusIndex::build(&scan).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 99)), Resolution::Absent),
            "an ID with no record and no parse failure must be Absent"
        );
    }

    #[test]
    fn parsed_record_wins_over_unrelated_diagnostic_on_same_file() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure(
            crate::parser::ParseFailureCause::TitleMissing,
            "docs/adr/cherry/CHE-0001-a.md",
        )];
        let scan = scanned(records, diags);
        let index = CorpusIndex::build(&scan).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 1)), Resolution::Resolved(_)),
            "a diagnostic on a file that still parsed must not mask the record"
        );
    }

    #[test]
    fn naming_violation_on_id_bearing_path_does_not_manufacture_indeterminate() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("CHE-0001-valid.md"),
            "# CHE-0001. Valid\n\nDate: 2026-04-29\nTier: B\nStatus: Accepted\n\n## Related\n\nRoot: CHE-0001\n\n## Context\n\nProse.\n",
        )
        .expect("write valid");
        std::fs::write(dir.path().join("CHE-0002-Bad_Name.md"), "# CHE-0002. Bad\n")
            .expect("write badly named");

        let domain_dir = crate::model::DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir.path().to_owned(),
        };
        let outcome = crate::parser::parse_domain(&domain_dir).expect("read_dir must succeed");

        assert!(
            outcome
                .diagnostics()
                .iter()
                .any(|d| d.file.contains("CHE-0002-Bad_Name.md")),
            "fixture must emit a diagnostic on the id-bearing path: {:?}",
            outcome.diagnostics()
        );

        let scan = ScannedCorpus::test_of(outcome);
        let index = CorpusIndex::build(&scan).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 2)), Resolution::Absent),
            "no file delivered CHE-0002, so it is absent — a naming diagnostic on an \
             id-bearing path must not manufacture an indeterminate"
        );
    }
}
