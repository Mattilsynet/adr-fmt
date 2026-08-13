//! Corpus-wide `AdrId -> AdrRecord` index.
//!
//! Per AFM-0008:R3 (permanent, globally unambiguous `PREFIX-NNNN`
//! identity) and audit finding F1, two records sharing an `AdrId` must
//! never resolve to an arbitrary survivor. [`CorpusIndex::build`] is
//! the single fallible pre-rule step every downstream consumer (link
//! checks, `--context`, `--refs`, `--tree`) shares: a corpus containing
//! a collision cannot produce a `CorpusIndex` at all, so no last-write-
//! wins map is reachable past this point (R16).

use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{AdrId, AdrRecord};

/// Two records claim the same `AdrId`. Carries both conflicting file
/// paths, sorted so the reported pair does not depend on which record
/// was scanned first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateId {
    pub id: AdrId,
    pub paths: [PathBuf; 2],
}

/// Corpus-wide `AdrId -> AdrRecord` lookup, built once as a pre-rule
/// step (audit finding F1).
#[derive(Debug)]
pub struct CorpusIndex<'a> {
    by_id: HashMap<&'a AdrId, &'a AdrRecord>,
}

impl<'a> CorpusIndex<'a> {
    /// Build the index over the full corpus.
    ///
    /// # Errors
    ///
    /// Returns [`DuplicateId`] when two records share an `AdrId`.
    /// `records` is sorted by `(prefix, number, file_path)` before
    /// indexing, so the detected collision — and the order of the two
    /// reported paths — is identical regardless of the order `records`
    /// arrives in.
    pub fn build(records: &'a [AdrRecord]) -> Result<Self, DuplicateId> {
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

        Ok(Self { by_id })
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
        let index = CorpusIndex::build(&records).expect("unique ids must build");
        assert!(index.contains_key(&make_id("CHE", 1)));
        assert!(index.contains_key(&make_id("CHE", 2)));
        assert!(!index.contains_key(&make_id("CHE", 99)));
    }

    #[test]
    fn build_detects_duplicate_within_one_directory() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md");

        let err = CorpusIndex::build(&[a, b]).expect_err("duplicate id must be rejected");
        assert_eq!(err.id, make_id("CHE", 1));
        assert_eq!(
            err.paths,
            [
                PathBuf::from("docs/adr/cherry/CHE-0001-a.md"),
                PathBuf::from("docs/adr/cherry/CHE-0001-b.md"),
            ]
        );
    }

    /// Load-bearing property (audit F1): the reported collision must
    /// not depend on which record the directory scan visited first.
    #[test]
    fn build_is_scan_order_independent() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-b.md");

        let forward =
            CorpusIndex::build(&[a.clone(), b.clone()]).expect_err("duplicate id must be rejected");
        let reversed = CorpusIndex::build(&[b, a]).expect_err("duplicate id must be rejected");

        assert_eq!(
            forward, reversed,
            "duplicate diagnostic must be identical regardless of scan order"
        );
    }

    #[test]
    fn build_detects_duplicate_across_two_directories() {
        let a = make_record("CHE", 1, "docs/adr/cherry/CHE-0001-a.md");
        let b = make_record("CHE", 1, "docs/adr/common/CHE-0001-b.md");

        let err = CorpusIndex::build(&[a, b]).expect_err("duplicate id must be rejected");
        assert_eq!(err.id, make_id("CHE", 1));
    }
}
