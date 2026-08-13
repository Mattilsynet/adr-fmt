//! Rule modules — each validates one aspect of ADR compliance.

mod links;
mod naming;
mod template;

use crate::config::Config;
use crate::index::CorpusIndex;
use crate::model::AdrRecord;
use crate::report::Diagnostic;

/// Run all rule modules and collect diagnostics.
///
/// `index` is the pre-validated [`CorpusIndex`] built once, before any
/// rule runs (audit F1 / AFM-0008:R3) — no rule module builds its own
/// corpus-wide `AdrId` map.
#[must_use]
pub fn run_all(records: &[AdrRecord], config: &Config, index: &CorpusIndex<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let domain_prefixes: Vec<&str> = config.domains.iter().map(|d| d.prefix.as_str()).collect();

    for record in records {
        template::check(record, config, &mut diagnostics);
        naming::check(record, &domain_prefixes, &mut diagnostics);
    }

    links::check(records, index, &mut diagnostics);

    diagnostics.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    diagnostics
}
