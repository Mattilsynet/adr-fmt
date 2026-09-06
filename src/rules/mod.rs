//! Rule modules — each validates one aspect of ADR compliance.

pub(crate) mod catalog;
mod links;
pub(crate) mod naming;
pub(crate) mod template;

use crate::config::Config;
use crate::index::CorpusIndex;
use crate::model::AdrRecord;
use crate::report::{DiagnosticSource, SourcedDiagnostic, document_diagnostics};

#[must_use]
pub fn run_all(
    records: &[AdrRecord],
    config: &Config,
    index: &CorpusIndex<'_>,
) -> Vec<SourcedDiagnostic> {
    let mut diagnostics = Vec::new();

    let domain_prefixes: Vec<&str> = config.domains.iter().map(|d| d.prefix.as_str()).collect();

    let budgets = template::Budgets::resolve(config, &mut diagnostics);
    let mut sourced: Vec<_> = diagnostics
        .drain(..)
        .map(|diagnostic| SourcedDiagnostic {
            source: DiagnosticSource::Global,
            diagnostic,
        })
        .collect();

    for record in records {
        template::check(record, config, &budgets, &mut diagnostics);
        naming::check(record, &domain_prefixes, &mut diagnostics);
        sourced.extend(document_diagnostics(
            record.file_path(),
            std::mem::take(&mut diagnostics),
        ));
    }

    links::check_sourced(
        records,
        index,
        links::GovernedPrefixes::new(&domain_prefixes),
        &mut sourced,
    );

    sourced
}
