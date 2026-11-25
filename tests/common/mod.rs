use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            rust_sources(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .expect("ADR directory is readable")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    out.sort();
    out
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub struct Corpus {
    pub live: BTreeMap<String, BTreeSet<u32>>,
    pub stale: BTreeSet<String>,
}

impl Corpus {
    #[allow(
        dead_code,
        reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
    )]
    pub fn load() -> Self {
        let root = manifest_dir().join("docs").join("adr");
        let id_re = Regex::new(r"^(AFM-\d{4})").expect("ADR filename pattern compiles");
        let rule_re = Regex::new(r"(?m)^R(\d+)\b").expect("corpus rule-line pattern compiles");

        let mut live = BTreeMap::new();
        for path in markdown_files(&root.join("adr-fmt")) {
            let name = path
                .file_name()
                .expect("ADR file has a name")
                .to_string_lossy()
                .into_owned();
            let Some(id) = id_re.captures(&name) else {
                continue;
            };
            let text = fs::read_to_string(&path).expect("ADR file is readable");
            let rules = rule_re
                .captures_iter(&text)
                .map(|caps| caps[1].parse::<u32>().expect("rule digits fit a u32"))
                .collect();
            live.insert(id[1].to_owned(), rules);
        }

        let mut stale = BTreeSet::new();
        for path in markdown_files(&root.join("stale")) {
            let name = path
                .file_name()
                .expect("ADR file has a name")
                .to_string_lossy()
                .into_owned();
            if let Some(id) = id_re.captures(&name) {
                stale.insert(id[1].to_owned());
            }
        }

        Self { live, stale }
    }
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub struct Citation {
    pub site: String,
    pub adr: String,
    pub rules: BTreeSet<u32>,
    pub text: String,
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub struct Patterns {
    pub adr: Regex,
    pub suffix: Regex,
    pub more: Regex,
    pub residual: Regex,
    pub malformed: Regex,
    pub continuation: Regex,
}

impl Patterns {
    #[allow(
        dead_code,
        reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
    )]
    pub fn new() -> Self {
        Self {
            adr: Regex::new(r"AFM-\d{4}").expect("citation pattern compiles"),
            suffix: Regex::new(r"^[: ]R(\d+)((?:\s*(?:,|/|-|\x{2013}|and)\s*R\d+)*)")
                .expect("rule-suffix pattern compiles"),
            more: Regex::new(r"\s*(,|/|-|\x{2013}|and)\s*R(\d+)")
                .expect("multi-rule pattern compiles"),
            residual: Regex::new(r"^\s*(?:,|;|/|&|-|\x{2013}|and)\s*R?\d")
                .expect("residual rule pattern compiles"),
            malformed: Regex::new(r"^[: ]R").expect("malformed suffix pattern compiles"),
            continuation: Regex::new(r"^\s*R\d").expect("continuation pattern compiles"),
        }
    }
}

impl Default for Patterns {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub struct Extracted {
    pub citations: Vec<(String, BTreeSet<u32>)>,
    pub trailing_bare_adr: bool,
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn extract_citations(text: &str, patterns: &Patterns) -> Result<Extracted, String> {
    let body = text.trim_end();
    let mut citations = Vec::new();
    let mut trailing_bare_adr = false;

    for caps in patterns.adr.captures_iter(body) {
        let whole = caps.get(0).expect("match zero always exists");
        let rest = &body[whole.end()..];
        let mut rules = BTreeSet::new();

        let consumed = if let Some(tail) = patterns.suffix.captures(rest) {
            let first: u32 = tail[1]
                .parse()
                .map_err(|_| format!("rule id `R{}` does not fit a u32", &tail[1]))?;
            rules.insert(first);
            let mut previous = first;
            for extra in patterns.more.captures_iter(&tail[2]) {
                let next: u32 = extra[2]
                    .parse()
                    .map_err(|_| format!("rule id `R{}` does not fit a u32", &extra[2]))?;
                if matches!(&extra[1], "-" | "\u{2013}") {
                    rules.extend(previous.min(next)..=previous.max(next));
                } else {
                    rules.insert(next);
                }
                previous = next;
            }
            tail.get(0).expect("match zero always exists").end()
        } else {
            if patterns.malformed.is_match(rest) {
                return Err(format!(
                    "`{}` is followed by `{}`, which opens a rule reference this guard cannot \
                     parse. A rule id it cannot read is a rule id it cannot check, and reporting \
                     it clean would be a false clean. AFM-0037:R4 forbids passing on unreadable \
                     input",
                    &caps[0],
                    rest.chars().take(16).collect::<String>()
                ));
            }
            trailing_bare_adr = rest.is_empty();
            0
        };

        let remainder = &rest[consumed..];
        if patterns.residual.is_match(remainder) {
            return Err(format!(
                "`{}` carries a rule suffix followed by `{}`, which looks like a further rule \
                 reference in a form this guard does not parse. Verifying part of a citation and \
                 silently dropping the rest is a false clean. AFM-0037:R4 forbids passing on \
                 unreadable input",
                &caps[0],
                remainder.chars().take(16).collect::<String>()
            ));
        }

        if consumed > 0 {
            trailing_bare_adr = false;
        }
        citations.push((caps[0].to_owned(), rules));
    }

    Ok(Extracted {
        citations,
        trailing_bare_adr,
    })
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn unresolved_citation_findings(citations: &[Citation], corpus: &Corpus) -> Vec<String> {
    let mut findings = Vec::new();
    for citation in citations {
        let Some(rules) = corpus.live.get(&citation.adr) else {
            if corpus.stale.contains(&citation.adr) {
                findings.push(format!(
                    "{}: cites {}, which is retired to docs/adr/stale/. Per AFM-0022 a stale \
                     ADR is a non-authoritative pointer and carries no binding rule: {}",
                    citation.site, citation.adr, citation.text
                ));
            } else {
                findings.push(format!(
                    "{}: cites {}, which is no ADR in this corpus, live or stale: {}",
                    citation.site, citation.adr, citation.text
                ));
            }
            continue;
        };
        for rule in &citation.rules {
            if !rules.contains(rule) {
                findings.push(format!(
                    "{}: cites {}:R{rule}, but {} declares no rule R{rule}; it declares {:?}: {}",
                    citation.site, citation.adr, citation.adr, rules, citation.text
                ));
            }
        }
    }
    findings
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub const COVERAGE_RESIDUAL: &str = "\
Existence is checked for every extracted citation, and so for every rule a citation names. \
Coverage is narrower: a rule is covered only when some citation text contains its catalog id \
as a literal whole token, so coverage is reachability by name, not a rule-reference claim. A \
bare citation that merely names the rule in prose covers it, and a citation that mentions an \
already-declared rule counts even when it governs nothing. Citations that name no catalog id \
reach no coverage set and gain only the existence check, so a scanner regression that stops \
reading just those citations is not caught here. DECLARED_COVERAGE is a deliberate contract: \
never delete an entry to make the guard pass; shrink-to-pass is a review-visible act.";

/// Parses catalog ids out of `entry("…")` and `wrapped("…")` construction only. A new
/// `RuleEntry` construction form must extend this parser, otherwise its ids are invisible to
/// coverage.
#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn catalog_rule_ids() -> BTreeSet<String> {
    let path = manifest_dir().join("src").join("rules").join("catalog.rs");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: catalog is readable: {e}", path.display()));
    let pattern =
        Regex::new(r#"\b(?:entry|wrapped)\(\s*"([^"]+)""#).expect("catalog entry pattern compiles");
    let ids: BTreeSet<String> = pattern
        .captures_iter(&text)
        .map(|caps| caps[1].to_owned())
        .collect();
    assert!(
        !ids.is_empty(),
        "{}: no `entry(\"...\")` or `wrapped(\"...\")` id was parsed. An empty catalog makes \
         per-rule coverage vacuous, so this guard fails rather than passing on nothing",
        path.display()
    );
    ids
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn rules_named_in(text: &str, catalog: &BTreeSet<String>) -> BTreeSet<String> {
    catalog
        .iter()
        .filter(|id| names_whole_token(text, id))
        .cloned()
        .collect()
}

fn names_whole_token(text: &str, id: &str) -> bool {
    let mut searched = 0;
    while let Some(offset) = text[searched..].find(id) {
        let start = searched + offset;
        let end = start + id.len();
        let before_is_boundary = text[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let after_is_boundary = text[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if before_is_boundary && after_is_boundary {
            return true;
        }
        searched = start + 1;
    }
    false
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn covered_rules(citations: &[Citation], catalog: &BTreeSet<String>) -> BTreeSet<String> {
    citations
        .iter()
        .flat_map(|citation| rules_named_in(&citation.text, catalog))
        .collect()
}

#[allow(
    dead_code,
    reason = "shared integration-test fixture helper; used by a subset of the test binaries, so it silences dead_code in a binary that does not use it and stays harmless where it does"
)]
pub fn assert_rule_coverage(
    kind: &str,
    declared: &[&str],
    citations: &[Citation],
    catalog: &BTreeSet<String>,
) {
    let computed = covered_rules(citations, catalog);
    let declared_set: BTreeSet<String> = declared.iter().copied().map(str::to_owned).collect();

    let unknown: Vec<&String> = declared_set.difference(catalog).collect();
    assert!(
        unknown.is_empty(),
        "declared {kind} coverage rule(s) that name no catalog RuleEntry: {unknown:?}. A \
         declared id that is not in src/rules/catalog.rs is a typo, a retired rule, or a \
         decorative id; no citation can ever cover it"
    );

    let uncited: Vec<&String> = declared_set.difference(&computed).collect();
    assert!(
        uncited.is_empty(),
        "declared {kind} coverage rule(s) with no citation naming them: {uncited:?}; computed \
         {computed:?}. Either the scanner stopped reaching text it used to read, or the last \
         truthful citation of this rule was removed. Repair the scanner, or remove the \
         declaration deliberately; an empty per-rule map is the vacuity this guard replaces the \
         count floor to catch"
    );

    let undeclared: Vec<&String> = computed.difference(&declared_set).collect();
    assert!(
        undeclared.is_empty(),
        "{kind} citations now name rule(s) with no declaration: {undeclared:?}; computed \
         {computed:?}. New citation-bearing text appeared, so pin exactly the rules it covers \
         and no others"
    );
}
