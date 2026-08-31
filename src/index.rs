use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{AdrId, AdrRecord, parse_adr_id_from_filename_stem};
use crate::report::Diagnostic;

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
    pub fn build(
        records: &'a [AdrRecord],
        parse_diagnostics: &[Diagnostic],
    ) -> Result<Self, DuplicateId> {
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

        let unparsed = collect_unparsed(parse_diagnostics, &by_id);

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
    parse_diagnostics: &[Diagnostic],
    by_id: &HashMap<&AdrId, &AdrRecord>,
) -> HashMap<AdrId, UnparsedTarget> {
    let mut unparsed: HashMap<AdrId, UnparsedTarget> = HashMap::new();

    for diagnostic in parse_diagnostics {
        let Some(id) = std::path::Path::new(&diagnostic.file)
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .and_then(parse_adr_id_from_filename_stem)
        else {
            continue;
        };
        if by_id.contains_key(&id) {
            continue;
        }
        unparsed.entry(id).or_insert_with(|| UnparsedTarget {
            path: diagnostic.file.clone(),
            rule: diagnostic.rule,
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
        *record.tier_mut() = Some(Tier::B);
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
        let index = CorpusIndex::build(&records, &[]).expect("unique ids must build");
        assert!(index.contains_key(&make_id("CHE", 1)));
        assert!(index.contains_key(&make_id("CHE", 2)));
        assert!(!index.contains_key(&make_id("CHE", 99)));
    }

    #[test]
    fn build_detects_duplicate_within_one_directory() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md");

        let err = CorpusIndex::build(&[a, b], &[]).expect_err("duplicate id must be rejected");
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

        let forward = CorpusIndex::build(&[a.clone(), b.clone()], &[])
            .expect_err("duplicate id must be rejected");
        let reversed = CorpusIndex::build(&[b, a], &[]).expect_err("duplicate id must be rejected");

        assert_eq!(
            forward, reversed,
            "duplicate diagnostic must be identical regardless of scan order"
        );
    }

    #[test]
    fn build_detects_duplicate_across_two_directories() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/common/CHE-0001-b.md");

        let err = CorpusIndex::build(&[a, b], &[]).expect_err("duplicate id must be rejected");
        assert_eq!(err.id, make_id("CHE", 1));
    }

    fn parse_failure(rule: &'static str, path: &str) -> Diagnostic {
        Diagnostic::warning(rule, std::path::Path::new(path), 0, "parse failed".into())
    }

    #[test]
    fn unparsed_target_resolves_indeterminate_not_absent() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure(
            "P002",
            "docs/adr/cherry/CHE-0002-broken-h1.md",
        )];
        let index = CorpusIndex::build(&records, &diags).expect("unique ids must build");

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
            "P002",
            "docs/adr/cherry/CHE-0002-broken-h1.md",
        )];
        let index = CorpusIndex::build(&records, &diags).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 99)), Resolution::Absent),
            "an ID with no record and no parse failure must be Absent"
        );
    }

    #[test]
    fn parsed_record_wins_over_unrelated_diagnostic_on_same_file() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure("P003", "docs/adr/cherry/CHE-0001-a.md")];
        let index = CorpusIndex::build(&records, &diags).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 1)), Resolution::Resolved(_)),
            "a diagnostic on a file that still parsed must not mask the record"
        );
    }

    #[test]
    fn diagnostic_on_non_adr_path_is_ignored() {
        let records = vec![make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md")];
        let diags = vec![parse_failure("P001", "docs/adr/cherry")];
        let index = CorpusIndex::build(&records, &diags).expect("unique ids must build");

        assert!(
            matches!(index.resolve(&make_id("CHE", 2)), Resolution::Absent),
            "a directory-level diagnostic yields no ADR id and must not create an indeterminate"
        );
    }
}
