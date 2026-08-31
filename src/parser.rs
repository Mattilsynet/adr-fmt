use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::model::{
    AdrId, AdrRecord, CrossDomainDefect, CrossDomainParent, DomainDir, MalformedReason, RelVerb,
    Related, Relationship, Status, TaggedRule, Tier, TierField, parse_adr_id,
    parse_adr_id_from_filename_stem,
};
use crate::report::Diagnostic;
use crate::rules::naming;

/// Records and diagnostics produced by parsing a directory of ADRs.
///
/// `records` contains successfully parsed ADRs ready for rule evaluation.
/// `diagnostics` carries per-file `P###` warnings (per AFM-0017) for
/// files whose filenames matched the prefix pattern but whose contents
/// could not be parsed. Infrastructure failures (unreadable directory)
/// surface as [`ParseError`] from the calling parser entrypoints.
#[derive(Debug, Default)]
pub struct ParseOutcome {
    records: Vec<AdrRecord>,
    diagnostics: Vec<Diagnostic>,
    parse_failures: Vec<FileParseFailure>,
}

impl ParseOutcome {
    #[must_use]
    pub fn records(&self) -> &[AdrRecord] {
        &self.records
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub(crate) fn into_parts(self) -> (Vec<AdrRecord>, Vec<Diagnostic>, Vec<FileParseFailure>) {
        (self.records, self.diagnostics, self.parse_failures)
    }

    #[cfg(test)]
    pub(crate) fn test_new(records: Vec<AdrRecord>, parse_failures: Vec<FileParseFailure>) -> Self {
        Self {
            records,
            diagnostics: Vec::new(),
            parse_failures,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileParseFailure {
    id: AdrId,
    path: PathBuf,
    cause: ParseFailureCause,
}

impl FileParseFailure {
    #[must_use]
    pub fn id(&self) -> &AdrId {
        &self.id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn rule(&self) -> &'static str {
        match self.cause {
            ParseFailureCause::Unreadable(_) => "P001",
            ParseFailureCause::TitleMissing => "P002",
        }
    }

    #[cfg(test)]
    pub(crate) fn test_new(id: AdrId, path: PathBuf, cause: ParseFailureCause) -> Self {
        Self { id, path, cause }
    }
}

/// Infrastructure failure from [`parse_domain`] or [`parse_stale`].
///
/// Both entrypoints have exactly one `Err` path: the target directory
/// itself could not be read via `fs::read_dir`. Per-file failures are
/// reported as `P001` diagnostics on the returned `ParseOutcome`
/// instead.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseError {
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ReadDir { path, source } => {
                write!(
                    f,
                    "cannot read domain/stale directory {}: {}",
                    path.display().to_string().escape_debug(),
                    source.to_string().escape_debug()
                )
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDir { source, .. } => Some(source),
        }
    }
}

/// Parse all ADR files in a domain directory.
///
/// Returns `Err` only when the domain directory itself cannot be
/// read; per-file failures are reported as `P###` diagnostics on
/// the returned `ParseOutcome`.
///
/// # Errors
///
/// Returns [`ParseError::ReadDir`] when `dir.path` cannot be read.
///
/// # Panics
///
/// Panics only if the internally-constructed filename regex is invalid.
pub fn parse_domain(dir: &DomainDir) -> Result<ParseOutcome, ParseError> {
    let entries = fs::read_dir(&dir.path).map_err(|e| ParseError::ReadDir {
        path: dir.path.clone(),
        source: e,
    })?;

    Ok(collect_domain_entries(dir, entries))
}

fn collect_domain_entries(
    dir: &DomainDir,
    entries: impl Iterator<Item = std::io::Result<fs::DirEntry>>,
) -> ParseOutcome {
    let mut outcome = ParseOutcome::default();

    let filename_re = Regex::new(&format!(
        r"^{}-(\d{{4}})-[a-z0-9]+(?:-[a-z0-9]+)*\.md$",
        regex::escape(&dir.prefix)
    ))
    .expect("valid regex");

    let known_prefixes = [dir.prefix.as_str()];

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                outcome.diagnostics.push(unreadable_entry(&dir.path, &e));
                continue;
            }
        };

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !is_adr_candidate(&name) {
            continue;
        }

        let path = entry.path();
        naming::check_file_name(&path, &known_prefixes, &mut outcome.diagnostics);

        if !filename_re.is_match(&name) {
            continue;
        }

        absorb_file(&mut outcome, &path, &dir.prefix, false);
    }

    outcome.records.sort_by(compare_record_order);
    outcome
}

fn compare_record_order(a: &AdrRecord, b: &AdrRecord) -> std::cmp::Ordering {
    (a.id().number(), a.id().prefix(), a.file_path()).cmp(&(
        b.id().number(),
        b.id().prefix(),
        b.file_path(),
    ))
}

fn is_adr_candidate(name: &str) -> bool {
    Path::new(name).extension().is_some_and(|ext| ext == "md") && name != "README.md"
}

fn unreadable_entry(dir: &Path, e: &std::io::Error) -> Diagnostic {
    Diagnostic::warning(
        "P001",
        dir,
        0,
        format!(
            "cannot read directory entry: {}",
            e.to_string().escape_debug()
        ),
    )
}

fn absorb_file(outcome: &mut ParseOutcome, path: &Path, prefix: &str, is_stale: bool) {
    match parse_adr_file(path, prefix, is_stale) {
        Ok(ParseFileOutcome::Parsed {
            record,
            diagnostics,
        }) => {
            outcome.records.push(*record);
            outcome.diagnostics.extend(diagnostics);
        }
        Ok(ParseFileOutcome::TitleMissing { diagnostics }) => {
            note_parse_failure(outcome, path, ParseFailureCause::TitleMissing);
            outcome.diagnostics.extend(diagnostics);
        }
        Err(e) => {
            let cause = ParseFailureCause::Unreadable(e);
            outcome
                .diagnostics
                .push(Diagnostic::warning("P001", path, 0, cause.to_string()));
            note_parse_failure(outcome, path, cause);
        }
    }
}

fn note_parse_failure(outcome: &mut ParseOutcome, path: &Path, cause: ParseFailureCause) {
    let Some(id) = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(parse_adr_id_from_filename_stem)
    else {
        return;
    };
    outcome.parse_failures.push(FileParseFailure {
        id,
        path: path.to_path_buf(),
        cause,
    });
}

/// Parse all ADR files in the stale directory.
///
/// Stale files may belong to any domain, so we try all configured
/// domain prefixes. Returns `Err` only when the stale directory
/// cannot be read.
///
/// # Errors
///
/// Returns [`ParseError::ReadDir`] when `stale_dir` cannot be read.
///
/// # Panics
///
/// Panics only if an internally-constructed domain filename regex is invalid.
pub fn parse_stale(stale_dir: &Path, config: &Config) -> Result<ParseOutcome, ParseError> {
    let entries = fs::read_dir(stale_dir).map_err(|e| ParseError::ReadDir {
        path: stale_dir.to_path_buf(),
        source: e,
    })?;

    Ok(collect_stale_entries(stale_dir, config, entries))
}

fn collect_stale_entries(
    stale_dir: &Path,
    config: &Config,
    entries: impl Iterator<Item = std::io::Result<fs::DirEntry>>,
) -> ParseOutcome {
    let mut outcome = ParseOutcome::default();

    let prefixes: Vec<(&str, Regex)> = config
        .domains
        .iter()
        .map(|d| {
            let pattern = format!(
                r"^{}-\d{{4}}-[a-z0-9]+(?:-[a-z0-9]+)*\.md$",
                regex::escape(&d.prefix),
            );
            (
                d.prefix.as_str(),
                Regex::new(&pattern).expect("valid regex"),
            )
        })
        .collect();

    let known_prefixes: Vec<&str> = prefixes.iter().map(|(p, _)| *p).collect();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                outcome.diagnostics.push(unreadable_entry(stale_dir, &e));
                continue;
            }
        };

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !is_adr_candidate(&name) {
            continue;
        }

        let path = entry.path();
        naming::check_file_name(&path, &known_prefixes, &mut outcome.diagnostics);

        for (prefix, re) in &prefixes {
            if re.is_match(&name) {
                absorb_file(&mut outcome, &path, prefix, true);
                break;
            }
        }
    }

    outcome.records.sort_by(compare_record_order);
    outcome
}

#[derive(Debug)]
pub(crate) enum ParseFileOutcome {
    Parsed {
        record: Box<AdrRecord>,
        diagnostics: Vec<Diagnostic>,
    },
    TitleMissing {
        diagnostics: Vec<Diagnostic>,
    },
}

#[derive(Debug)]
pub(crate) enum ParseFailureCause {
    Unreadable(ReadFileError),
    TitleMissing,
}

#[cfg(test)]
impl ParseFailureCause {
    pub(crate) fn test_unreadable(path: PathBuf) -> Self {
        Self::Unreadable(ReadFileError {
            path,
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        })
    }
}

impl core::fmt::Display for ParseFailureCause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unreadable(source) => source.fmt(f),
            Self::TitleMissing => f.write_str("missing or malformed H1 title"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReadFileError {
    path: PathBuf,
    source: std::io::Error,
}

impl ReadFileError {
    fn reason(&self) -> String {
        match self.source.kind() {
            std::io::ErrorKind::NotFound => "file not found".to_string(),
            std::io::ErrorKind::PermissionDenied => "permission denied".to_string(),
            _ => self.source.to_string().escape_debug().to_string(),
        }
    }
}

impl core::fmt::Display for ReadFileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "cannot read ADR file {}: {}",
            self.path.display().to_string().escape_debug(),
            self.reason()
        )
    }
}

impl std::error::Error for ReadFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub(crate) fn parse_adr_file(
    path: &Path,
    expected_prefix: &str,
    is_stale: bool,
) -> Result<ParseFileOutcome, ReadFileError> {
    let content = fs::read_to_string(path).map_err(|source| ReadFileError {
        path: path.to_path_buf(),
        source,
    })?;
    let lines: Vec<&str> = content.lines().collect();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let source = SourceLines::scan(&lines);
    let outside = source.outside();

    if lines.is_empty() {
        diagnostics.push(Diagnostic::warning(
            "P002",
            path,
            0,
            format!(
                "ADR file is empty; expected `# {}-NNNN. Title` H1",
                expected_prefix.escape_debug()
            ),
        ));
        return Ok(ParseFileOutcome::TitleMissing { diagnostics });
    }

    let Some((id, title, title_line)) = parse_title(&outside, expected_prefix) else {
        diagnostics.push(Diagnostic::warning(
            "P002",
            path,
            0,
            format!(
                "missing or malformed H1 title; expected `# {}-NNNN. Title text`",
                expected_prefix.escape_debug()
            ),
        ));
        return Ok(ParseFileOutcome::TitleMissing { diagnostics });
    };

    let (date, _) = find_field(&lines, "Date:");
    let (last_reviewed, _) = find_field(&lines, "Last-reviewed:");
    let (tier, _) = find_tier_field(&lines);

    let (status, status_line, status_raw) = find_status_field(&lines);
    let status_from_section = status.is_none() && has_heading(&outside, "Status");

    let (related, related_diagnostics) = find_relationships(&outside, path);
    diagnostics.extend(related_diagnostics);

    let has_context = has_heading(&outside, "Context");
    let has_decision = has_heading(&outside, "Decision");
    let has_consequences = has_heading(&outside, "Consequences");
    let has_retirement = has_heading(&outside, "Retirement");

    let (section_order, section_word_counts) = analyze_sections(&outside);

    let crates = find_crates_field(&lines);

    let parent_cross_domain = find_parent_cross_domain_field(&lines);

    let decision_rules = extract_tagged_rules(&source.rule_scan());

    let (max_code_block_lines, max_code_block_line) = measure_code_blocks(&source);

    let record = Box::new(AdrRecord::from_parser_fields(
        id,
        path.to_owned(),
        Some(title),
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
    ));
    Ok(ParseFileOutcome::Parsed {
        record,
        diagnostics,
    })
}

fn parse_title(
    outside: &OutsideLines<'_>,
    expected_prefix: &str,
) -> Option<(AdrId, String, usize)> {
    for (line_no, line) in outside.iter() {
        if let Some(rest) = line.strip_prefix("# ")
            && let Some(dot_pos) = rest.find(". ")
            && let Some(id) = parse_adr_id(&rest[..dot_pos])
            && id.prefix() == expected_prefix
        {
            let title = rest[dot_pos + 2..].trim();
            if title.is_empty() {
                continue;
            }
            return Some((id, title.to_owned(), line_no));
        }
    }
    None
}

fn find_field(lines: &[&str], key: &str) -> (Option<String>, usize) {
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") {
            break;
        }
        if let Some(value) = line.strip_prefix(key) {
            let value = value.trim();
            if !value.is_empty() {
                return (Some(value.to_owned()), i + 1);
            }
        }
    }
    (None, 0)
}

fn find_tier_field(lines: &[&str]) -> (TierField, usize) {
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") {
            break;
        }
        if let Some(value) = line.strip_prefix("Tier:") {
            let value = value.trim();
            let field = match Tier::parse(value) {
                Some(tier) => TierField::Valid(tier),
                None => TierField::Invalid {
                    raw: value.to_owned(),
                },
            };
            return (field, i + 1);
        }
    }
    (TierField::Absent, 0)
}

fn find_status_field(lines: &[&str]) -> (Option<Status>, usize, Option<String>) {
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") {
            break;
        }
        if let Some(value) = line.strip_prefix("Status:") {
            let value = value.trim();
            if !value.is_empty() {
                let raw = value.to_owned();
                let status = Status::parse(value);
                return (Some(status), i + 1, Some(raw));
            }
        }
    }
    (None, 0, None)
}

fn find_relationships(outside: &OutsideLines<'_>, path: &Path) -> (Related, Vec<Diagnostic>) {
    let mut rels = Vec::new();
    let mut in_related = false;
    let mut found_section = false;
    let mut malformed: Option<(String, MalformedReason)> = None;
    let mut diagnostics = Vec::new();

    for (line_no, line) in outside.iter() {
        if line == "## Related" {
            in_related = true;
            found_section = true;
            continue;
        }
        if !in_related {
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if line.starts_with("## ") {
            break;
        }
        let trimmed = line.trim();
        if trimmed == "—" || trimmed == "- —" {
            continue;
        }
        for segment in trimmed.split(" | ") {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }
            parse_related_segment(
                segment,
                line,
                line_no,
                path,
                &mut rels,
                &mut malformed,
                &mut diagnostics,
            );
        }
    }

    let related = match malformed {
        Some((line, reason)) => Related::Malformed {
            line,
            reason,
            relationships: rels,
        },
        None if found_section => Related::Parsed(rels),
        None => Related::Absent,
    };

    (related, diagnostics)
}

fn parse_related_segment(
    segment: &str,
    line: &str,
    line_no: usize,
    path: &Path,
    rels: &mut Vec<Relationship>,
    malformed: &mut Option<(String, MalformedReason)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(colon_pos) = segment.find(": ") else {
        diagnostics.push(Diagnostic::warning(
            "P003",
            path,
            line_no,
            format!("malformed `## Related` segment (missing `Verb: ` separator): `{segment}`"),
        ));
        malformed.get_or_insert_with(|| (line.to_owned(), MalformedReason::MissingSeparator));
        return;
    };
    let verb_str = &segment[..colon_pos];
    let targets_str = &segment[colon_pos + 2..];

    let Some(verb) = RelVerb::parse(verb_str) else {
        diagnostics.push(Diagnostic::warning(
            "P003",
            path,
            line_no,
            format!("malformed `## Related` segment (unrecognized verb `{verb_str}`): `{segment}`"),
        ));
        malformed.get_or_insert_with(|| {
            (
                line.to_owned(),
                MalformedReason::UnknownVerb(verb_str.to_owned()),
            )
        });
        return;
    };

    for target_str in targets_str.split(", ") {
        let clean = strip_annotation(target_str);
        if let Some(target_id) = parse_adr_id(clean) {
            rels.push(Relationship {
                verb,
                target: target_id,
                line: line_no,
            });
        } else {
            diagnostics.push(Diagnostic::warning(
                "P003",
                path,
                line_no,
                format!(
                    "malformed `## Related` segment (unparseable target `{target_str}`): \
                     `{segment}`"
                ),
            ));
            malformed.get_or_insert_with(|| {
                (
                    line.to_owned(),
                    MalformedReason::UnparseableTarget(target_str.trim().to_owned()),
                )
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineKind {
    FenceOpen,
    FenceClose,
    Inside,
    Outside,
}

#[derive(Debug, Default)]
struct FenceScanner {
    inside: bool,
}

impl FenceScanner {
    fn classify(&mut self, line: &str) -> LineKind {
        if line.starts_with("```") {
            self.inside = !self.inside;
            if self.inside {
                LineKind::FenceOpen
            } else {
                LineKind::FenceClose
            }
        } else if self.inside {
            LineKind::Inside
        } else {
            LineKind::Outside
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClassifiedLine<'a> {
    line_no: usize,
    text: &'a str,
    kind: LineKind,
}

#[derive(Debug)]
struct SourceLines<'a> {
    classified: Vec<ClassifiedLine<'a>>,
}

#[derive(Debug)]
struct OutsideLines<'a> {
    lines: Vec<(usize, &'a str)>,
}

#[derive(Debug)]
struct RuleScanLines<'a> {
    lines: Vec<RuleScanLine<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleScanLine<'a> {
    Outside { line_no: usize, text: &'a str },
    FenceBlock,
}

impl<'a> SourceLines<'a> {
    fn scan(lines: &[&'a str]) -> Self {
        let mut scanner = FenceScanner::default();
        let classified = lines
            .iter()
            .enumerate()
            .map(|(i, line)| ClassifiedLine {
                line_no: i + 1,
                text: line,
                kind: scanner.classify(line),
            })
            .collect();
        Self { classified }
    }

    fn outside(&self) -> OutsideLines<'a> {
        OutsideLines {
            lines: self
                .classified
                .iter()
                .filter(|c| c.kind == LineKind::Outside)
                .map(|c| (c.line_no, c.text))
                .collect(),
        }
    }

    fn rule_scan(&self) -> RuleScanLines<'a> {
        RuleScanLines {
            lines: self
                .classified
                .iter()
                .filter_map(|c| match c.kind {
                    LineKind::Outside => Some(RuleScanLine::Outside {
                        line_no: c.line_no,
                        text: c.text,
                    }),
                    LineKind::FenceOpen => Some(RuleScanLine::FenceBlock),
                    LineKind::FenceClose | LineKind::Inside => None,
                })
                .collect(),
        }
    }

    fn classified(&self) -> impl Iterator<Item = ClassifiedLine<'a>> + '_ {
        self.classified.iter().copied()
    }
}

impl<'a> OutsideLines<'a> {
    fn iter(&self) -> impl Iterator<Item = (usize, &'a str)> + '_ {
        self.lines.iter().copied()
    }
}

impl<'a> RuleScanLines<'a> {
    fn as_slice(&self) -> &[RuleScanLine<'a>] {
        &self.lines
    }
}

fn analyze_sections(outside: &OutsideLines<'_>) -> (Vec<String>, HashMap<String, usize>) {
    let mut order = Vec::new();
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    let mut current_section: Option<String> = None;
    let mut current_words = 0usize;

    for (_, line) in outside.iter() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(ref section) = current_section {
                *word_counts.entry(section.clone()).or_insert(0) += current_words;
            }

            let name = heading.trim().to_owned();
            order.push(name.clone());
            current_section = Some(name);
            current_words = 0;
        } else if current_section.is_some() && !line.is_empty() {
            current_words += line.split_whitespace().count();
        }
    }

    if let Some(ref section) = current_section {
        *word_counts.entry(section.clone()).or_insert(0) += current_words;
    }

    (order, word_counts)
}

fn find_crates_field(lines: &[&str]) -> Vec<String> {
    for line in lines {
        if let Some(value) = line.strip_prefix("Crates:") {
            let value = value.trim();
            if value.is_empty() {
                return Vec::new();
            }
            return value.split(',').map(|s| s.trim().to_owned()).collect();
        }
        if line.starts_with("## ") {
            break;
        }
    }
    Vec::new()
}

fn find_parent_cross_domain_field(lines: &[&str]) -> CrossDomainParent {
    for line in lines {
        if line.starts_with("## ") {
            break;
        }
        if let Some(value) = line.strip_prefix("Parent-cross-domain:") {
            let value = value.trim();
            if value.is_empty() {
                return CrossDomainParent::Malformed {
                    raw: String::new(),
                    reason: CrossDomainDefect::EmptyField,
                };
            }
            let (id_part, reason) = split_id_and_reason(value);
            let Some(id) = parse_adr_id(id_part.trim()) else {
                return CrossDomainParent::Malformed {
                    raw: value.to_owned(),
                    reason: CrossDomainDefect::UnparseableId(id_part.trim().to_owned()),
                };
            };
            let reason = reason.trim();
            if reason.is_empty() {
                return CrossDomainParent::Malformed {
                    raw: value.to_owned(),
                    reason: CrossDomainDefect::MissingReason,
                };
            }
            return CrossDomainParent::Valid {
                id,
                reason: reason.to_owned(),
            };
        }
    }
    CrossDomainParent::Absent
}

fn split_id_and_reason(value: &str) -> (&str, &str) {
    if let Some(idx) = value.find('—') {
        return (&value[..idx], &value[idx + '—'.len_utf8()..]);
    }
    if let Some(idx) = value.find(" - ") {
        return (&value[..idx], &value[idx + 3..]);
    }
    if let Some(idx) = value.find(char::is_whitespace) {
        return (&value[..idx], &value[idx..]);
    }
    (value, "")
}

static TAGGED_RULE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^R(\d+)\s*\[(\d+)\]:\s*(.+)").expect("valid regex"));

fn extract_tagged_rules(scan: &RuleScanLines<'_>) -> Vec<TaggedRule> {
    let scanned = scan.as_slice();
    let mut rules = Vec::new();
    let mut in_decision = false;

    let mut i = 0;
    while i < scanned.len() {
        let RuleScanLine::Outside {
            line_no,
            text: line,
        } = scanned[i]
        else {
            i += 1;
            continue;
        };
        if line == "## Decision" {
            in_decision = true;
            i += 1;
            continue;
        }
        if !in_decision {
            i += 1;
            continue;
        }
        if line.starts_with("## ") {
            break;
        }
        if let Some((_, [num, layer_str, rule_text])) =
            TAGGED_RULE_RE.captures(line).map(|caps| caps.extract())
        {
            let layer: u8 = layer_str.parse().unwrap_or(0);
            let mut text = rule_text.trim().to_owned();
            let rule_line = line_no;

            i += 1;
            while i < scanned.len() {
                let RuleScanLine::Outside { text: next, .. } = scanned[i] else {
                    break;
                };
                if next.trim().is_empty() {
                    break;
                }
                if next.starts_with("## ") {
                    break;
                }
                if TAGGED_RULE_RE.is_match(next) {
                    break;
                }
                if next.len() >= 2 && next.starts_with("  ") {
                    text.push(' ');
                    text.push_str(next.trim());
                    i += 1;
                } else {
                    break;
                }
            }

            rules.push(TaggedRule {
                id: format!("R{num}"),
                text,
                layer,
                line: rule_line,
            });
        } else {
            i += 1;
        }
    }

    rules
}

fn strip_annotation(s: &str) -> &str {
    let s = s.trim();
    if let Some(paren_start) = s.find(" (") {
        s[..paren_start].trim()
    } else {
        s
    }
}

fn measure_code_blocks(source: &SourceLines<'_>) -> (usize, usize) {
    let mut current_lines = 0usize;
    let mut current_start = 0usize;
    let mut max_lines = 0usize;
    let mut max_start = 0usize;
    let mut unclosed = false;

    for line in source.classified() {
        match line.kind {
            LineKind::FenceOpen => {
                current_lines = 0;
                current_start = line.line_no;
                unclosed = true;
            }
            LineKind::FenceClose => {
                unclosed = false;
                if current_lines > max_lines {
                    max_lines = current_lines;
                    max_start = current_start;
                }
            }
            LineKind::Inside => current_lines += 1,
            LineKind::Outside => {}
        }
    }

    if unclosed && current_lines > max_lines {
        max_lines = current_lines;
        max_start = current_start;
    }

    (max_lines, max_start)
}

fn has_heading(outside: &OutsideLines<'_>, name: &str) -> bool {
    let target = format!("## {name}");
    outside.iter().any(|(_, line)| line == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_of<'a>(lines: &[&'a str]) -> SourceLines<'a> {
        SourceLines::scan(lines)
    }

    fn outside_of<'a>(lines: &[&'a str]) -> OutsideLines<'a> {
        SourceLines::scan(lines).outside()
    }

    fn rule_scan_of<'a>(lines: &[&'a str]) -> RuleScanLines<'a> {
        SourceLines::scan(lines).rule_scan()
    }

    #[test]
    fn parse_title_extracts_id_and_text() {
        let lines = vec![
            "# CHE-0042. Event Envelope Construction Invariants",
            "",
            "Date: 2026-04-25",
        ];
        let (id, title, line) = parse_title(&outside_of(&lines), "CHE").unwrap();
        assert_eq!(id.prefix(), "CHE");
        assert_eq!(id.number(), 42);
        assert_eq!(title, "Event Envelope Construction Invariants");
        assert_eq!(line, 1);
    }

    #[test]
    fn parse_title_wrong_prefix_returns_none() {
        let lines = vec!["# PAR-0001. Some Title"];
        assert!(parse_title(&outside_of(&lines), "CHE").is_none());
    }

    #[test]
    fn find_field_extracts_date() {
        let lines = vec![
            "# Title",
            "",
            "Date: 2026-04-25",
            "Last-reviewed: 2026-04-25",
        ];
        let (date, line) = find_field(&lines, "Date:");
        assert_eq!(date.as_deref(), Some("2026-04-25"));
        assert_eq!(line, 3);
    }

    #[test]
    fn find_relationships_parses_multi_target() {
        let lines = vec![
            "## Related",
            "",
            "References: CHE-0006, CHE-0032 | Supersedes: CHE-0015",
            "",
            "## Context",
        ];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert_eq!(rels.len(), 3);
        assert_eq!(rels[0].verb, RelVerb::References);
        assert_eq!(rels[0].target, parse_adr_id("CHE-0006").unwrap());
        assert_eq!(rels[1].target, parse_adr_id("CHE-0032").unwrap());
        assert_eq!(rels[2].verb, RelVerb::Supersedes);
        assert_eq!(rels[2].target, parse_adr_id("CHE-0015").unwrap());
    }

    #[test]
    fn find_relationships_parses_root_verb() {
        let lines = vec!["## Related", "", "Root: CHE-0001", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].verb, RelVerb::Root);
        assert_eq!(rels[0].target.prefix(), "CHE");
        assert_eq!(rels[0].target.number(), 1);
    }

    #[test]
    fn strip_annotation_removes_parenthetical() {
        assert_eq!(strip_annotation("CHE-0036 (indirect)"), "CHE-0036");
        assert_eq!(strip_annotation("CHE-0021"), "CHE-0021");
        assert_eq!(
            strip_annotation("CHE-0021 (`#[non_exhaustive]`)"),
            "CHE-0021"
        );
    }

    #[test]
    fn has_heading_finds_section() {
        let lines = vec![
            "## Status",
            "",
            "Accepted",
            "",
            "## Context",
            "",
            "Some text.",
        ];
        assert!(has_heading(&outside_of(&lines), "Context"));
        assert!(!has_heading(&outside_of(&lines), "Decision"));
    }

    #[test]
    fn has_heading_finds_retirement() {
        let lines = vec!["## Retirement", "", "Deprecated because reasons."];
        assert!(has_heading(&outside_of(&lines), "Retirement"));
    }

    #[test]
    fn measure_code_blocks_counts_lines() {
        let lines = vec![
            "some text",
            "```rust",
            "fn main() {}",
            "let x = 1;",
            "let y = 2;",
            "```",
            "more text",
        ];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 3, "3 lines between fences");
        assert_eq!(start, 2, "opening fence is line 2 (1-indexed)");
    }

    #[test]
    fn measure_code_blocks_multiple_blocks() {
        let lines = vec![
            "```", "line1", "```", "text", "```rust", "a", "b", "c", "d", "e", "```",
        ];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 5, "second block has 5 lines");
        assert_eq!(start, 5, "second block opens at line 5 (1-indexed)");
    }

    #[test]
    fn measure_code_blocks_empty_block() {
        let lines = vec!["```", "```"];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 0);
        assert_eq!(start, 0);
    }

    #[test]
    fn measure_code_blocks_no_blocks() {
        let lines = vec!["some text", "more text"];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 0);
        assert_eq!(start, 0, "no blocks means start line 0");
    }

    #[test]
    fn measure_code_blocks_fence_lines_excluded() {
        let lines = vec!["```", "only_this", "```"];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 1, "only content line counted, not fences");
        assert_eq!(start, 1);
    }

    #[test]
    fn measure_code_blocks_unclosed_block() {
        let lines = vec!["text", "```rust", "fn main() {}", "let x = 1;"];
        let (max, start) = measure_code_blocks(&source_of(&lines));
        assert_eq!(max, 2, "unclosed block has 2 content lines");
        assert_eq!(start, 2, "opening fence at line 2 (1-indexed)");
    }

    #[test]
    fn find_relationships_detects_placeholder() {
        let lines = vec!["## Related", "", "- —", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        assert!(related.relationships().is_empty());
    }

    #[test]
    fn find_relationships_detects_bare_dash_placeholder() {
        let lines = vec!["## Related", "", "—", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        assert!(related.relationships().is_empty());
    }

    #[test]
    fn find_relationships_no_placeholder_with_rels() {
        let lines = vec!["## Related", "", "References: CHE-0001", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        assert_eq!(related.relationships().len(), 1);
    }

    #[test]
    fn analyze_sections_correct_order() {
        let lines = vec![
            "# CHE-0001. Title",
            "",
            "## Related",
            "",
            "Root: CHE-0001",
            "",
            "## Context",
            "",
            "This is the context with enough words to pass validation easily.",
            "",
            "## Decision",
            "",
            "We decided to do this thing because it makes sense to us.",
            "",
            "## Consequences",
            "",
            "This makes testing easier and code more maintainable overall.",
        ];
        let (order, counts) = analyze_sections(&outside_of(&lines));
        assert_eq!(
            order,
            vec!["Related", "Context", "Decision", "Consequences"]
        );
        assert_eq!(counts["Context"], 11);
        assert_eq!(counts["Decision"], 12);
        assert_eq!(counts["Consequences"], 9);
    }

    #[test]
    fn analyze_sections_excludes_code_blocks() {
        let lines = vec![
            "## Decision",
            "",
            "We decided to use this approach.",
            "",
            "```rust",
            "fn main() {",
            "    println!(\"hello\");",
            "}",
            "```",
            "",
            "That is all.",
        ];
        let (_, counts) = analyze_sections(&outside_of(&lines));
        assert_eq!(counts["Decision"], 9);
    }

    #[test]
    fn analyze_sections_with_retirement() {
        let lines = vec![
            "## Status",
            "",
            "Deprecated",
            "",
            "## Retirement",
            "",
            "Deprecated because the transport layer moved to a different protocol entirely.",
        ];
        let (order, counts) = analyze_sections(&outside_of(&lines));
        assert!(order.contains(&"Retirement".to_owned()));
        assert_eq!(counts["Retirement"], 11);
    }

    #[test]
    fn duplicate_h2_accumulates_word_count_instead_of_overwriting() {
        let long_prose = ["word"; 250].join(" ");
        let body = format!(
            "# CHE-0001. Duplicate Context\n\n\
             Date: 2026-04-27\nLast-reviewed: 2026-04-27\nTier: S\nStatus: Accepted\n\n\
             ## Related\n\nRoot: CHE-0001\n\n\
             ## Context\n\n{long_prose}\n\n\
             ## Context\n\nToo short.\n\n\
             ## Decision\n\nR1 [5]: We decided a thing for reasons that are written out here.\n"
        );
        let outcome = parse_markdown("CHE-0001-duplicate-context.md", &body);
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };

        assert_eq!(
            record.section_order(),
            ["Related", "Context", "Context", "Decision"],
            "every H2 occurrence stays visible in section order"
        );
        assert_eq!(
            record.section_word_counts().get("Context"),
            Some(&252),
            "prose under a repeated `## Context` accumulates; a short duplicate must not \
             conceal the earlier section's word count from T015"
        );
    }

    #[test]
    fn self_referencing_detected() {
        let lines = vec!["## Related", "", "Root: CHE-0001", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        let rels = related.relationships();
        let id = AdrId::test_new("CHE", 1);
        let is_self_ref = rels
            .iter()
            .any(|rel| rel.verb == RelVerb::Root && rel.target == id);
        assert!(is_self_ref);
    }

    #[test]
    fn self_referencing_wrong_id_not_detected() {
        let lines = vec!["## Related", "", "Root: CHE-0002", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        let rels = related.relationships();
        let id = AdrId::test_new("CHE", 1);
        let is_self_ref = rels
            .iter()
            .any(|rel| rel.verb == RelVerb::Root && rel.target == id);
        assert!(!is_self_ref);
    }

    #[test]
    fn find_crates_field_present() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Date: 2026-04-25",
            "Crates: example-core, example-gateway",
            "Tier: A",
            "",
            "## Status",
        ];
        let crates = find_crates_field(&lines);
        assert_eq!(crates, vec!["example-core", "example-gateway"]);
    }

    #[test]
    fn find_crates_field_empty() {
        let lines = vec!["# CHE-0042. Title", "", "Crates:", "", "## Status"];
        let crates = find_crates_field(&lines);
        assert!(crates.is_empty());
    }

    #[test]
    fn find_crates_field_absent() {
        let lines = vec!["# CHE-0042. Title", "", "Date: 2026-04-25", "", "## Status"];
        let crates = find_crates_field(&lines);
        assert!(crates.is_empty());
    }

    #[test]
    fn find_parent_cross_domain_em_dash() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: COM-0001 — bridges principle to architecture",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        let CrossDomainParent::Valid { id, reason } = &field else {
            panic!("expected a valid declaration, got: {field:?}");
        };
        assert_eq!(id.prefix(), "COM");
        assert_eq!(id.number(), 1);
        assert_eq!(reason, "bridges principle to architecture");
    }

    #[test]
    fn find_parent_cross_domain_ascii_hyphen() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: COM-0001 - reason text",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        let CrossDomainParent::Valid { reason, .. } = &field else {
            panic!("expected a valid declaration, got: {field:?}");
        };
        assert_eq!(reason, "reason text");
    }

    #[test]
    fn parent_cross_domain_separator_is_any_whitespace_not_only_a_dash() {
        for (separator, line) in [
            (
                "plain space",
                "Parent-cross-domain: COM-0001 arbitrary text",
            ),
            ("tab", "Parent-cross-domain: COM-0001\tarbitrary text"),
            (
                "non-breaking space",
                "Parent-cross-domain: COM-0001\u{a0}arbitrary text",
            ),
        ] {
            let lines = vec!["# CHE-0042. Title", "", line, "", "## Status"];
            let field = find_parent_cross_domain_field(&lines);
            let CrossDomainParent::Valid { id, reason } = &field else {
                panic!("expected {separator} to separate id from reason, got: {field:?}");
            };
            assert_eq!(id.prefix(), "COM", "prefix under {separator}");
            assert_eq!(id.number(), 1, "number under {separator}");
            assert_eq!(reason, "arbitrary text", "reason under {separator}");
        }
    }

    #[test]
    fn find_parent_cross_domain_id_only_is_malformed_not_valid() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: COM-0001",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(
                &field,
                CrossDomainParent::Malformed {
                    reason: CrossDomainDefect::MissingReason,
                    ..
                }
            ),
            "AFM-0020:R3 requires `PREFIX-NNNN — reason`; an ID-only \
             declaration must not parse as valid, got: {field:?}"
        );
        assert!(
            field.honoured_id().is_none(),
            "a reasonless declaration must carry no suppression authority"
        );
    }

    #[test]
    fn find_parent_cross_domain_empty_reason_after_dash_is_malformed() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: COM-0001 —   ",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(
                &field,
                CrossDomainParent::Malformed {
                    reason: CrossDomainDefect::MissingReason,
                    ..
                }
            ),
            "a whitespace-only reason must not count as a reason, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_absent() {
        let lines = vec!["# CHE-0042. Title", "", "Date: 2026-04-25", "", "## Status"];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(field, CrossDomainParent::Absent),
            "no field must parse as Absent, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_empty_field_is_malformed_not_absent() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain:",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(
                &field,
                CrossDomainParent::Malformed {
                    reason: CrossDomainDefect::EmptyField,
                    ..
                }
            ),
            "a present-but-empty field must be distinguishable from no field, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_invalid_id_is_malformed_not_absent() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: not-an-id — a reason",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(
                &field,
                CrossDomainParent::Malformed {
                    reason: CrossDomainDefect::UnparseableId(raw),
                    ..
                } if raw == "not-an-id"
            ),
            "an unparseable ID must be Malformed, not Absent, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_stops_at_h2() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "## Status",
            "",
            "Parent-cross-domain: COM-0001 — late field",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(field, CrossDomainParent::Absent),
            "a field below the preamble must not be read, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_garbage_after_colon() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: !!!@@@",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            field.honoured_id().is_none(),
            "garbage must grant no suppression, got: {field:?}"
        );
        assert!(
            matches!(field, CrossDomainParent::Malformed { .. }),
            "garbage must be Malformed, not Absent, got: {field:?}"
        );
    }

    #[test]
    fn find_parent_cross_domain_lowercase_prefix_rejected() {
        let lines = vec![
            "# CHE-0042. Title",
            "",
            "Parent-cross-domain: com-0001 — bad case",
            "",
            "## Status",
        ];
        let field = find_parent_cross_domain_field(&lines);
        assert!(
            matches!(field, CrossDomainParent::Malformed { .. }),
            "lowercase prefix must be rejected as malformed, got: {field:?}"
        );
    }

    #[test]
    fn extract_tagged_rules_normal() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [5]: All events must be versioned",
            "R2 [5]: Snapshots at 100-event intervals",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "R1");
        assert_eq!(rules[0].text, "All events must be versioned");
        assert_eq!(rules[0].layer, 5);
        assert_eq!(rules[1].id, "R2");
        assert_eq!(rules[1].text, "Snapshots at 100-event intervals");
        assert_eq!(rules[1].layer, 5);
    }

    #[test]
    fn extract_tagged_rules_no_rules_returns_empty() {
        let lines = vec![
            "## Decision",
            "",
            "We use event sourcing for persistence.",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert!(rules.is_empty());
    }

    #[test]
    fn extract_tagged_rules_mixed_with_prose() {
        let lines = vec![
            "## Decision",
            "",
            "We adopt the following rules:",
            "",
            "R1 [6]: Events are append-only",
            "Some prose between rules.",
            "R2 [6]: Snapshots are optional",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "R1");
        assert_eq!(rules[1].id, "R2");
    }

    #[test]
    fn extract_tagged_rules_malformed_ignored() {
        let lines = vec![
            "## Decision",
            "",
            "Rfoo [5]: Not a valid rule tag",
            "R1 [5]: Valid rule",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "R1");
    }

    #[test]
    fn extract_tagged_rules_multiline() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [5]: Construct EventEnvelope exclusively through",
            "  EventEnvelope::new(), which validates non-nil event_id",
            "  and returns Result<Self, EnvelopeError>",
            "R2 [5]: Use NonZeroU64 for the sequence field",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "R1");
        assert_eq!(
            rules[0].text,
            "Construct EventEnvelope exclusively through \
             EventEnvelope::new(), which validates non-nil event_id \
             and returns Result<Self, EnvelopeError>"
        );
        assert_eq!(rules[1].id, "R2");
        assert_eq!(rules[1].text, "Use NonZeroU64 for the sequence field");
    }

    #[test]
    fn extract_tagged_rules_blank_line_terminates_continuation() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [5]: First part of rule",
            "",
            "  This should NOT be continuation (after blank line)",
            "R2 [5]: Second rule",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].text, "First part of rule");
        assert_eq!(rules[1].text, "Second rule");
    }

    #[test]
    fn extract_tagged_rules_backtick_content_preserved() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [5]: Construct via `EventEnvelope::new()` which",
            "  returns `Result<Self, EnvelopeError>`",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].text,
            "Construct via `EventEnvelope::new()` which \
             returns `Result<Self, EnvelopeError>`"
        );
    }

    #[test]
    fn extract_tagged_rules_non_numeric_layer_is_not_a_rule() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [abc]: Non-numeric layer",
            "R2 []: Empty layer",
            "R3 [5]: Valid rule",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(
            rules.len(),
            1,
            "a non-numeric or empty layer fails the tag regex, so the line is \
             not recognised as a tagged rule at all: {rules:?}"
        );
        assert_eq!(rules[0].id, "R3");
    }

    #[test]
    fn extract_tagged_rules_layer_parsed() {
        let lines = vec![
            "## Decision",
            "",
            "R1 [1]: Paradigm-level rule",
            "R2 [12]: Parameter-level rule",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].layer, 1);
        assert_eq!(rules[1].layer, 12);
    }

    #[test]
    fn extract_tagged_rules_old_format_not_matched() {
        let lines = vec![
            "## Decision",
            "",
            "- **R1**: Old format rule",
            "- **R2**: Another old format rule",
            "",
            "## Consequences",
        ];
        let rules = extract_tagged_rules(&rule_scan_of(&lines));
        assert!(rules.is_empty(), "old format should not be parsed");
    }

    #[test]
    fn parse_adr_file_empty_file_emits_p002() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-empty.md");
        fs::write(&path, "").expect("write empty file");

        let outcome = parse_adr_file(&path, "CHE", false).expect("read should succeed");
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("no record from empty file")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
        assert!(
            diagnostics[0].message.contains("empty"),
            "message should mention empty"
        );
    }

    #[test]
    fn missing_h1_outcome_carries_no_record_slot_at_all() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-no-h1.md");
        fs::write(&path, "Some prose without an H1 title.\n").expect("write file");

        match parse_adr_file(&path, "CHE", false).expect("read should succeed") {
            ParseFileOutcome::TitleMissing { diagnostics } => {
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].rule, "P002");
            }
            ParseFileOutcome::Parsed { .. } => {
                panic!("a file without an H1 must not reach the parsed state")
            }
        }
    }

    #[test]
    fn unreadable_file_error_preserves_io_error_kind_through_the_source_chain() {
        let err = parse_adr_file(
            Path::new("/nonexistent/path/that/should/never/exist/CHE-0001.md"),
            "CHE",
            false,
        )
        .expect_err("missing file should bubble as Err");

        let source = std::error::Error::source(&err).expect("error must expose its io source");
        let io = source
            .downcast_ref::<std::io::Error>()
            .expect("source chain must carry the original io::Error");
        assert_eq!(
            io.kind(),
            std::io::ErrorKind::NotFound,
            "ErrorKind must survive the parse boundary"
        );
    }

    #[test]
    fn parse_adr_file_missing_h1_emits_p002() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-no-h1.md");
        fs::write(
            &path,
            "Some prose without an H1 title.\n\n## Status\n\nAccepted\n",
        )
        .expect("write file");

        let outcome = parse_adr_file(&path, "CHE", false).expect("read should succeed");
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("no record without H1")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
        assert!(
            diagnostics[0].message.contains("missing or malformed"),
            "message should mention malformed title"
        );
    }

    #[test]
    fn parse_adr_file_wrong_prefix_h1_emits_p002() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-wrong-prefix.md");
        fs::write(&path, "# PAR-0001. Wrong prefix\n").expect("write file");

        let outcome = parse_adr_file(&path, "CHE", false).expect("read should succeed");
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("no record when prefix doesn't match expected")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
    }

    #[test]
    fn parse_adr_file_h1_with_trailing_space_emits_p002() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-trailing-space.md");
        fs::write(&path, "# CHE-0001 . Title\n").expect("write file");

        let outcome = parse_adr_file(&path, "CHE", false).expect("read should succeed");
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("malformed H1 yields no record")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
    }

    #[test]
    fn parse_adr_file_valid_h1_emits_no_diagnostics() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("CHE-0001-valid.md");
        fs::write(
            &path,
            "# CHE-0001. Valid Title\n\nDate: 2026-04-29\nTier: B\nStatus: Accepted\n\n## Related\n\nRoot: CHE-0001\n\n## Context\n\nProse.\n",
        )
        .expect("write file");

        let outcome = parse_adr_file(&path, "CHE", false).expect("read should succeed");
        let ParseFileOutcome::Parsed { diagnostics, .. } = outcome else {
            panic!("record should be parsed")
        };
        assert!(
            diagnostics.is_empty(),
            "valid file should emit no diagnostics"
        );
    }

    #[test]
    fn parse_adr_file_unreadable_returns_err() {
        let outcome = parse_adr_file(
            Path::new("/nonexistent/path/that/should/never/exist/CHE-0001.md"),
            "CHE",
            false,
        );
        assert!(outcome.is_err(), "missing file should bubble as Err");
    }

    #[test]
    fn parse_domain_unreadable_dir_returns_err() {
        let domain_dir = DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: std::path::PathBuf::from(
                "/nonexistent/path/that/should/never/exist/for/parse-domain-test",
            ),
        };
        let outcome = parse_domain(&domain_dir);
        assert!(
            outcome.is_err(),
            "unreadable domain directory should bubble as Err per AFM-0017 R4"
        );
        let err = outcome.unwrap_err();
        assert!(matches!(err, ParseError::ReadDir { .. }), "got: {err:?}");
        assert!(
            err.to_string()
                .contains("cannot read domain/stale directory"),
            "error should describe the failure: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn parse_domain_preserves_read_error_kind_end_to_end() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::os::unix::fs::symlink(
            dir.path().join("CHE-0002-vanished-target.md"),
            dir.path().join("CHE-0002-dangling.md"),
        )
        .expect("create dangling symlink");

        let domain_dir = DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir.path().to_owned(),
        };
        let outcome = parse_domain(&domain_dir).expect("read_dir should succeed");

        assert_eq!(outcome.parse_failures.len(), 1, "one recorded failure");
        let failure = &outcome.parse_failures[0];
        assert_eq!(failure.rule(), "P001");

        let ParseFailureCause::Unreadable(err) = &failure.cause else {
            panic!("an unreadable file must record its typed read error")
        };
        let source = std::error::Error::source(err).expect("error must expose its io source");
        let io = source
            .downcast_ref::<std::io::Error>()
            .expect("source chain must carry the original io::Error");
        assert_eq!(
            io.kind(),
            std::io::ErrorKind::NotFound,
            "ErrorKind must survive all the way to the ParseOutcome boundary"
        );

        assert_eq!(outcome.diagnostics.len(), 1, "one P001 diagnostic");
        assert_eq!(outcome.diagnostics[0].rule, "P001");
        assert!(
            outcome.diagnostics[0].message.contains("file not found"),
            "the P001 message must be derived from the io ErrorKind, got: {}",
            outcome.diagnostics[0].message
        );
    }

    #[test]
    fn parse_domain_with_unreadable_file_emits_p001() {
        let dir = tempfile::tempdir().expect("tempdir");
        let valid_path = dir.path().join("CHE-0001-valid.md");
        fs::write(
            &valid_path,
            "# CHE-0001. Valid\n\nDate: 2026-04-29\nTier: B\nStatus: Accepted\n\n## Related\n\nRoot: CHE-0001\n\n## Context\n\nProse.\n",
        )
        .expect("write valid");

        fs::create_dir(dir.path().join("CHE-0002-actually-a-dir.md"))
            .expect("create masquerading dir");

        let domain_dir = DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir.path().to_owned(),
        };
        let outcome = parse_domain(&domain_dir).expect("read_dir should succeed");
        assert_eq!(outcome.records.len(), 1, "one valid record");
        assert_eq!(outcome.diagnostics.len(), 1, "one P001 diagnostic");
        assert_eq!(outcome.diagnostics[0].rule, "P001");
        assert!(
            outcome.diagnostics[0]
                .message
                .contains("cannot read ADR file"),
            "P001 message should describe failure"
        );
    }

    #[test]
    fn parse_domain_with_unreadable_dir_entry_emits_diagnostic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let domain_dir = DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir.path().to_owned(),
        };
        let entries = fs::read_dir(&domain_dir.path).expect("read_dir");
        let outcome = collect_domain_entries(&domain_dir, failing_entries(entries));
        assert_eq!(
            outcome.diagnostics.len(),
            1,
            "an unreadable directory entry is indeterminate, not absent"
        );
        assert_eq!(outcome.diagnostics[0].rule, "P001");
        assert!(
            outcome.diagnostics[0]
                .message
                .contains("cannot read directory entry"),
            "diagnostic should name the entry failure: {}",
            outcome.diagnostics[0].message
        );
    }

    fn failing_entries(
        _real: fs::ReadDir,
    ) -> impl Iterator<Item = std::io::Result<fs::DirEntry>> + use<> {
        std::iter::once(Err(std::io::Error::other("simulated entry failure")))
    }

    #[test]
    fn directory_level_p001_on_id_bearing_dir_does_not_manufacture_indeterminate() {
        let root = tempfile::tempdir().expect("tempdir");
        let dir_path = root.path().join("CHE-0002-domain");
        fs::create_dir(&dir_path).expect("create id-bearing domain dir");

        let domain_dir = DomainDir {
            prefix: "CHE".to_string(),
            name: "Cherry-pit Test".to_string(),
            path: dir_path,
        };
        let entries = fs::read_dir(&domain_dir.path).expect("read_dir");
        let outcome = collect_domain_entries(&domain_dir, failing_entries(entries));

        assert_eq!(outcome.diagnostics.len(), 1, "one directory-level P001");
        assert_eq!(outcome.diagnostics[0].rule, "P001");
        assert!(
            outcome.diagnostics[0].file.contains("CHE-0002-domain"),
            "fixture must put an id-bearing path on the diagnostic: {}",
            outcome.diagnostics[0].file
        );

        let scan = crate::index::ScannedCorpus::test_of(outcome);
        let index =
            crate::index::CorpusIndex::build(&scan).expect("no records, so build must succeed");

        assert!(
            matches!(
                index.resolve(&AdrId::test_new("CHE", 2)),
                crate::index::Resolution::Absent
            ),
            "no ADR file claimed CHE-0002 — a directory-level P001 must not \
             manufacture an indeterminate"
        );
    }

    #[test]
    fn parse_stale_with_unreadable_file_emits_p001() {
        let dir = tempfile::tempdir().expect("tempdir");

        fs::create_dir(dir.path().join("CHE-0099-actually-a-dir.md"))
            .expect("create masquerading dir");

        let config_toml = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Example"
directory = "example"
description = "Test domain"
crates = []
"#;
        let config: Config = toml::from_str(config_toml).expect("parse config");

        let outcome = parse_stale(dir.path(), &config).expect("read_dir should succeed");
        assert!(outcome.records.is_empty(), "no valid records");
        assert_eq!(outcome.diagnostics.len(), 1, "one P001 diagnostic");
        assert_eq!(outcome.diagnostics[0].rule, "P001");
        assert!(
            outcome.diagnostics[0]
                .message
                .contains("cannot read ADR file"),
            "P001 message should describe failure"
        );
    }

    #[test]
    fn find_status_field_parses_superseded_with_target() {
        let lines = vec![
            "# CHE-0001. Title",
            "Date: 2026-04-27",
            "Tier: B",
            "Status: Superseded by CHE-0099",
            "",
            "## Related",
        ];
        let (status, _, raw) = find_status_field(&lines);
        assert!(matches!(status, Some(Status::SupersededBy(_))));
        assert_eq!(raw.as_deref(), Some("Superseded by CHE-0099"));
    }

    #[test]
    fn find_status_field_none_when_absent() {
        let lines = vec![
            "# CHE-0001. Title",
            "Date: 2026-04-27",
            "Tier: B",
            "",
            "## Related",
        ];
        let (status, _, _) = find_status_field(&lines);
        assert_eq!(status, None);
    }

    #[test]
    fn find_status_field_stops_at_h2() {
        let lines = vec![
            "# CHE-0001. Title",
            "Date: 2026-04-27",
            "Tier: B",
            "",
            "## Context",
            "Status: Accepted",
        ];
        let (status, _, _) = find_status_field(&lines);
        assert_eq!(status, None);
    }

    #[test]
    fn find_relationships_empty_section_returns_no_rels() {
        let lines = vec!["## Related", "", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert!(rels.is_empty());
    }

    #[test]
    fn find_relationships_old_bullet_format_not_parsed() {
        let lines = vec![
            "## Related",
            "",
            "- Root: CHE-0001",
            "- References: CHE-0002",
            "",
            "## Context",
        ];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert!(rels.is_empty());
    }

    #[test]
    fn find_relationships_single_verb_no_pipe() {
        let lines = vec!["## Related", "", "References: CHE-0005", "", "## Context"];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].verb, RelVerb::References);
        assert_eq!(rels[0].target.number(), 5);
    }

    #[test]
    fn find_relationships_whitespace_around_pipe() {
        let lines = vec![
            "## Related",
            "",
            "Root: CHE-0001  |  References: CHE-0002 ,  CHE-0003",
            "",
            "## Context",
        ];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert_eq!(rels.len(), 3);
        assert_eq!(rels[0].verb, RelVerb::Root);
        assert_eq!(rels[1].target.number(), 2);
        assert_eq!(rels[2].target.number(), 3);
    }

    #[test]
    fn find_relationships_no_space_pipe_not_parsed() {
        let lines = vec![
            "## Related",
            "",
            "Root: CHE-0001|References: CHE-0002",
            "",
            "## Context",
        ];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert!(
            rels.is_empty(),
            "no-space pipe should not split into segments and strict ID parser \
             rejects trailing text"
        );
    }

    #[test]
    fn find_relationships_multi_line_content() {
        let lines = vec![
            "## Related",
            "",
            "Root: CHE-0001",
            "References: CHE-0002, CHE-0003",
            "",
            "## Context",
        ];
        let (related, _diags) = find_relationships(&outside_of(&lines), Path::new("test.md"));
        assert!(!matches!(related, Related::Absent));
        let rels = related.relationships();
        assert_eq!(rels.len(), 3);
        assert_eq!(rels[0].verb, RelVerb::Root);
        assert_eq!(rels[1].verb, RelVerb::References);
        assert_eq!(rels[1].target.number(), 2);
        assert_eq!(rels[2].verb, RelVerb::References);
        assert_eq!(rels[2].target.number(), 3);
    }

    #[test]
    fn find_field_stops_at_h2_boundary() {
        let lines = vec![
            "# CHE-0001. Title",
            "",
            "## Context",
            "",
            "Date: 2026-01-01",
        ];
        let (date, _) = find_field(&lines, "Date:");
        assert_eq!(
            date, None,
            "Date: inside a section body should not be found"
        );
    }

    #[test]
    fn find_tier_field_stops_at_h2_boundary() {
        let lines = vec![
            "# CHE-0001. Title",
            "",
            "Tier: B",
            "",
            "## Context",
            "",
            "Tier: A",
        ];
        let (tier, line) = find_tier_field(&lines);
        assert_eq!(
            tier.value(),
            Some(crate::model::Tier::B),
            "should find preamble tier"
        );
        assert_eq!(line, 3);

        let lines_after = vec!["# CHE-0001. Title", "", "## Context", "", "Tier: A"];
        let (tier2, _) = find_tier_field(&lines_after);
        assert!(
            matches!(tier2, TierField::Absent),
            "Tier: inside a section body should not be found, got: {tier2:?}"
        );
    }

    #[test]
    fn find_tier_field_invalid_value_is_not_absent() {
        let lines = vec!["# CHE-0001. Title", "", "Tier: Z", ""];
        let (tier, line) = find_tier_field(&lines);
        assert!(
            matches!(&tier, TierField::Invalid { raw } if raw == "Z"),
            "an unrecognized Tier value must be Invalid, not Absent, got: {tier:?}"
        );
        assert_eq!(line, 3);
    }

    #[test]
    fn find_tier_field_empty_value_is_invalid_not_absent() {
        let lines = vec!["# CHE-0001. Title", "", "Tier:", ""];
        let (tier, _) = find_tier_field(&lines);
        assert!(
            matches!(&tier, TierField::Invalid { raw } if raw.is_empty()),
            "a present-but-empty Tier field must be Invalid, not Absent, got: {tier:?}"
        );
    }

    fn parse_markdown(name: &str, body: &str) -> ParseFileOutcome {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(name);
        fs::write(&path, body).expect("write file");
        parse_adr_file(&path, "CHE", false).expect("read should succeed")
    }

    #[test]
    fn fenced_h1_does_not_manufacture_a_title() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-h1.md",
            "Guidance for authors.\n\n```markdown\n# CHE-0001. Fabricated Title\n```\n",
        );
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("an H1 that exists only inside a fence must not produce a record")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
        assert_eq!(diagnostics[0].line, 0, "AFM-0017:R3 pins P002 at line 0");
    }

    #[test]
    fn fenced_h1_does_not_shift_the_real_title_line() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-then-real.md",
            "Preamble prose.\n\n```markdown\n# CHE-0001. Fabricated Title\n```\n\n# CHE-0001. Real Title\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("real H1 must parse")
        };
        assert_eq!(record.title(), Some("Real Title"));
        assert_eq!(
            record.title_line(),
            7,
            "line number must be the real H1 line, not the fenced one"
        );
    }

    #[test]
    fn fenced_related_section_does_not_manufacture_edges() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-related.md",
            "# CHE-0001. Fence Related\n\n## Context\n\nEvery ADR declares lineage like this:\n\n```markdown\n## Related\n\nReferences: CHE-0009\n```\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        assert!(
            matches!(record.related(), Related::Absent),
            "a fenced `## Related` example must not register as a real section, got: {:?}",
            record.related()
        );
    }

    #[test]
    fn fenced_related_entry_does_not_add_an_edge_or_shift_lines() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-related-entry.md",
            "# CHE-0001. Fence Related Entry\n\n## Related\n\nRoot: CHE-0001\n\n```markdown\nReferences: CHE-0009\n```\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        let Related::Parsed(rels) = record.related() else {
            panic!(
                "expected a parsed Related section, got: {:?}",
                record.related()
            );
        };
        assert_eq!(
            rels.len(),
            1,
            "only the unfenced entry is real, got: {rels:?}"
        );
        assert_eq!(rels[0].target.number(), 1);
        assert_eq!(rels[0].line, 5, "line number must survive fence gating");
    }

    #[test]
    fn fenced_tagged_rule_does_not_register_as_normative() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-rule.md",
            "# CHE-0001. Fence Rules\n\n## Decision\n\nR1 [5]: The real normative rule.\n\nAuthors tag rules like this:\n\n```markdown\nR2 [3]: An illustrative example rule.\n```\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        let rules = record.decision_rules();
        assert_eq!(
            rules.len(),
            1,
            "only the unfenced rule is normative, got: {rules:?}"
        );
        assert_eq!(rules[0].id, "R1");
        assert_eq!(
            rules[0].line, 5,
            "rule line number must survive fence gating"
        );
    }

    #[test]
    fn fenced_lines_do_not_join_as_rule_continuations() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-continuation.md",
            "# CHE-0001. Fence Continuation\n\n## Decision\n\nR1 [5]: The real rule.\n```markdown\n  not a continuation\n```\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        let rules = record.decision_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].text, "The real rule.",
            "fenced lines must not be joined into the rule text"
        );
    }

    #[test]
    fn prose_after_a_fence_does_not_join_the_preceding_rule() {
        let outcome = parse_markdown(
            "CHE-0001-post-fence-prose.md",
            "# CHE-0001. Post Fence Prose\n\n## Decision\n\nR1 [5]: The real rule.\n\
             ```markdown\n  fenced example line\n```\n  outside prose after the fence.\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        let rules = record.decision_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].text, "The real rule.",
            "a fence block terminates continuation; post-fence prose must not be joined"
        );
    }

    #[test]
    fn fenced_heading_does_not_satisfy_a_required_section() {
        let outcome = parse_markdown(
            "CHE-0001-fenced-heading.md",
            "# CHE-0001. Fence Headings\n\n## Decision\n\nThe template requires:\n\n```markdown\n## Context\n```\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        assert!(
            !record.has_context(),
            "a fenced `## Context` must not satisfy the required section"
        );
        assert!(
            record.has_decision(),
            "the real `## Decision` heading must still be seen"
        );
    }

    #[test]
    fn tilde_fences_are_documented_as_unsupported() {
        let outcome = parse_markdown(
            "CHE-0001-tilde-fence.md",
            "# CHE-0001. Tilde Fence\n\n## Decision\n\n~~~markdown\nR9 [3]: Tilde-fenced example rule.\n~~~\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        assert_eq!(
            record.decision_rules().len(),
            1,
            "known limitation: only ``` fences are recognized (AFM-0006:R3); \
             ~~~ fences are deliberately NOT inert"
        );
    }

    #[test]
    fn four_space_indented_code_blocks_are_not_inert() {
        let outcome = parse_markdown(
            "CHE-0001-indented-block.md",
            "# CHE-0001. Indented Block\n\n## Context\n\nAn indented block follows.\n\n    one two three four\n    five six\n\n## Decision\n\n    ## Consequences\n",
        );
        let ParseFileOutcome::Parsed { record, .. } = outcome else {
            panic!("record should parse")
        };
        assert_eq!(
            record.max_code_block_lines(),
            0,
            "AFM-0006:R3: an indented block is not a recognized fenced block"
        );
        assert_eq!(
            record.section_word_counts().get("Context"),
            Some(&10),
            "AFM-0006:R3: indented lines stay live prose and are still word-counted"
        );
        assert!(
            !record.has_consequences(),
            "an indented `## Consequences` is not an ATX heading (AFM-0006:R3)"
        );
        assert_eq!(
            record.section_order(),
            ["Context", "Decision"],
            "indented content must not add or suppress a section"
        );
    }

    #[test]
    fn empty_h1_title_emits_p002_at_line_zero() {
        let outcome = parse_markdown(
            "CHE-0001-empty-title.md",
            "# CHE-0001. \n\n## Context\n\nProse.\n",
        );
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("an H1 with no title text is not a valid title")
        };
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule, "P002");
        assert_eq!(diagnostics[0].line, 0, "AFM-0017:R3 pins P002 at line 0");
    }

    #[test]
    fn whitespace_only_h1_title_emits_p002() {
        let outcome = parse_markdown(
            "CHE-0001-blank-title.md",
            "# CHE-0001.    \n\n## Context\n\nProse.\n",
        );
        let ParseFileOutcome::TitleMissing { diagnostics } = outcome else {
            panic!("a whitespace-only title is not a valid title")
        };
        assert_eq!(diagnostics[0].rule, "P002");
    }

    fn write_adr(dir: &Path, file_name: &str, id: &str, title: &str) {
        let body = format!(
            "# {id}. {title}\n\n\
             Date: 2026-04-27\nLast-reviewed: 2026-04-27\nTier: S\nStatus: Accepted\n\n\
             ## Related\n\nRoot: {id}\n\n\
             ## Context\n\nProse explaining why this record exists at all.\n\n\
             ## Decision\n\nR1 [5]: We decided a thing for stated reasons.\n"
        );
        fs::write(dir.join(file_name), body).expect("write ADR fixture");
    }

    fn entries_in_order(dir: &Path, names: &[&str]) -> Vec<fs::DirEntry> {
        let mut found: Vec<Option<fs::DirEntry>> = fs::read_dir(dir)
            .expect("read fixture dir")
            .map(|e| Some(e.expect("dir entry")))
            .collect();

        names
            .iter()
            .map(|name| {
                let at = found
                    .iter()
                    .position(|e| {
                        e.as_ref()
                            .is_some_and(|e| e.file_name().to_string_lossy() == **name)
                    })
                    .expect("fixture present");
                found[at].take().expect("fixture not yet consumed")
            })
            .collect()
    }

    fn file_names_of(outcome: &ParseOutcome) -> Vec<String> {
        outcome
            .records
            .iter()
            .map(|r| {
                r.file_path()
                    .file_name()
                    .expect("record has a file name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }

    #[test]
    fn domain_record_order_is_independent_of_directory_enumeration_order() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        write_adr(path, "CHE-0001-alpha.md", "CHE-0001", "Alpha");
        write_adr(path, "CHE-0001-beta.md", "CHE-0001", "Beta");
        write_adr(path, "CHE-0002-gamma.md", "CHE-0002", "Gamma");

        let dir = DomainDir {
            path: path.to_path_buf(),
            prefix: "CHE".to_owned(),
            name: "cherry".to_owned(),
        };

        let permutations = [
            ["CHE-0001-alpha.md", "CHE-0001-beta.md", "CHE-0002-gamma.md"],
            ["CHE-0001-beta.md", "CHE-0001-alpha.md", "CHE-0002-gamma.md"],
            ["CHE-0002-gamma.md", "CHE-0001-beta.md", "CHE-0001-alpha.md"],
            ["CHE-0001-beta.md", "CHE-0002-gamma.md", "CHE-0001-alpha.md"],
        ];

        let orders: Vec<Vec<String>> = permutations
            .iter()
            .map(|names| {
                let entries = entries_in_order(path, names);
                file_names_of(&collect_domain_entries(&dir, entries.into_iter().map(Ok)))
            })
            .collect();

        for order in &orders {
            assert_eq!(
                order, &orders[0],
                "records sharing a number must land in a fixed order regardless of \
                 the order the directory enumerated them in"
            );
        }
        assert_eq!(
            orders[0],
            ["CHE-0001-alpha.md", "CHE-0001-beta.md", "CHE-0002-gamma.md"],
            "the tie between equal numbers breaks on file path"
        );
    }

    #[test]
    fn stale_record_order_is_independent_of_directory_enumeration_order() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test domain"
crates = []

[[domains]]
prefix = "DEC"
name = "Deco"
directory = "deco"
description = "Test domain"
crates = []
"#;
        let config: Config = toml::from_str(toml_str).expect("valid config");

        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path();
        write_adr(path, "CHE-0001-alpha.md", "CHE-0001", "Alpha");
        write_adr(path, "DEC-0001-beta.md", "DEC-0001", "Beta");
        write_adr(path, "CHE-0002-gamma.md", "CHE-0002", "Gamma");

        let permutations = [
            ["CHE-0001-alpha.md", "DEC-0001-beta.md", "CHE-0002-gamma.md"],
            ["DEC-0001-beta.md", "CHE-0001-alpha.md", "CHE-0002-gamma.md"],
            ["CHE-0002-gamma.md", "DEC-0001-beta.md", "CHE-0001-alpha.md"],
        ];

        let orders: Vec<Vec<String>> = permutations
            .iter()
            .map(|names| {
                let entries = entries_in_order(path, names);
                file_names_of(&collect_stale_entries(
                    path,
                    &config,
                    entries.into_iter().map(Ok),
                ))
            })
            .collect();

        for order in &orders {
            assert_eq!(
                order, &orders[0],
                "stale records sharing a number across prefixes must land in a fixed \
                 order regardless of directory enumeration order"
            );
        }
        assert_eq!(
            orders[0],
            ["CHE-0001-alpha.md", "DEC-0001-beta.md", "CHE-0002-gamma.md"],
            "the tie between equal numbers breaks on prefix, then file path"
        );
    }
}
