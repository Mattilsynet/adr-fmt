//! Bidirectional parity guard between the diagnostics adr-fmt implements
//! and the rule registry its governance output renders.
//!
//! The rendered side is read from the real binary's stdout, never from
//! source text: a source-scanning guard is satisfied by an id appearing in
//! a comment or a test string and so fails open.
//!
//! The implemented side no longer scans for string literals. Every
//! diagnostic is now constructed through its `src/rules/catalog.rs` entry
//! (`catalog::T002.diagnostic(..)`), which is also the only place in the
//! crate that turns a severity into a `Diagnostic`, so this guard resolves
//! those references through the catalog. That keeps the check honest in both directions: an
//! entry a validator emits but no section renders is caught, and a rendered
//! id no validator emits is caught. It also gives the guard a stronger
//! invariant than before — a bare literal rule id at a construction site is
//! now a failure, not merely invisible.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

const PARITY_CONFIG: &str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "TST"
name = "Test Domain"
directory = "test"
description = "Parity guard domain."
crates = []
"#;

const CONSTRUCTION_MARKER: &str = ".diagnostic(";

const DIAGNOSTIC_TYPE: &str = "Diagnostic";

/// Every way to name a `Diagnostic` constructor, WITHOUT a trailing `(`.
///
/// Requiring the parenthesis would miss `let make = Diagnostic::warning;`,
/// which binds the constructor as a value and calls it later.
const DIAGNOSTIC_CONSTRUCTORS: [&str; 2] = ["Diagnostic::warning", "Diagnostic::error"];

/// Tokens that put `Diagnostic` in type position, where a following `{`
/// opens a body or a declaration rather than a struct literal.
const TYPE_POSITION_TOKENS: [&str; 5] = ["->", "struct", "impl", "enum", "for"];

/// The file that DEFINES `Diagnostic`, exempt from the construction ban.
const DEFINITION_FILE: &str = "report.rs";

const FORWARDING_MARKER: &str = "resolve_param(";

const FORWARDING_DEFINITION: &str = "fn resolve_param(";

const FORWARDING_DEFINITION_FILE: &str = "rules/template.rs";

const FORWARDED_PARAM_NAME: &str = "rule";

const FORWARDING_ARGUMENT_PREFIX: &str = "&catalog::";

const CATALOG_FILE: &str = "rules/catalog.rs";

const CATALOG_PATH_PREFIX: &str = "catalog::";

fn is_rule_id(token: &str) -> bool {
    match token.as_bytes() {
        [head, a, b, c] => {
            head.is_ascii_uppercase()
                && a.is_ascii_digit()
                && b.is_ascii_digit()
                && c.is_ascii_digit()
        }
        [head, a, b, c, tail] => {
            head.is_ascii_uppercase()
                && a.is_ascii_digit()
                && b.is_ascii_digit()
                && c.is_ascii_digit()
                && tail.is_ascii_lowercase()
        }
        _ => false,
    }
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("src directory is readable") {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

struct ProductionSegment<'a> {
    offset: usize,
    text: &'a str,
}

fn inline_test_module_end(src: &str, body: usize) -> usize {
    src[body..]
        .find("\n}\n")
        .map_or(src.len(), |at| body + at + 3)
}

fn production_segments(src: &str) -> Vec<ProductionSegment<'_>> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut cursor = 0;
    while let Some(offset) = src[cursor..].find("\n#[cfg(test)]\n") {
        let attribute = cursor + offset + 1;
        let body = attribute + "#[cfg(test)]\n".len();
        if src[body..].starts_with("mod ") {
            segments.push(ProductionSegment {
                offset: start,
                text: &src[start..attribute],
            });
            let end = inline_test_module_end(src, body);
            start = end;
            cursor = end;
        } else {
            cursor = attribute + 1;
        }
    }
    segments.push(ProductionSegment {
        offset: start,
        text: &src[start..],
    });
    segments
}

fn forwarding_definition_span(production: &str, file: &Path) -> Option<(usize, usize)> {
    let matches = production.match_indices(FORWARDING_DEFINITION).count();
    assert!(
        matches <= 1,
        "{}: found {matches} definitions of `{FORWARDING_DEFINITION}`; the parity guard \
         exempts exactly one forwarding definition, so a second one must not be added \
         without re-proving the exemption",
        file.display()
    );
    let start = production.find(FORWARDING_DEFINITION)?;
    let end = production[start..]
        .find("\n}\n")
        .map_or(production.len(), |at| start + at + 2);
    Some((start, end))
}

fn leading_identifier(production: &str, site: usize) -> &str {
    let rest = production[site..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    &rest[..end]
}

/// Map every catalog constant name to the rule id it carries.
///
/// Parsed from source rather than imported because `catalog` is crate-private
/// (AFM-0026:R2) and an integration test cannot see it. The constant name and
/// the id deliberately differ in case for `T005c`, so the mapping must be read
/// rather than guessed.
fn catalog_entries() -> BTreeMap<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(CATALOG_FILE);
    let text = fs::read_to_string(&path).expect("catalog source is readable");
    let mut entries = BTreeMap::new();
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find("pub(crate) const ") {
        let name_start = cursor + offset + "pub(crate) const ".len();
        let name_end = text[name_start..]
            .find(':')
            .map(|at| name_start + at)
            .expect("a catalog constant declares a type");
        let name = text[name_start..name_end].trim().to_string();
        cursor = name_end;
        let declared_type = text[name_end + 1..]
            .trim_start()
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .next()
            .unwrap_or_default();
        if declared_type != "RuleEntry" {
            continue;
        }
        let Some(open) = text[cursor..].find('"').map(|at| cursor + at + 1) else {
            break;
        };
        let close = text[open..]
            .find('"')
            .map(|at| open + at)
            .expect("unterminated catalog string literal");
        let id = &text[open..close];
        if is_rule_id(id) {
            entries.insert(name, id.to_string());
        }
        cursor = close;
    }
    assert!(
        entries.len() >= 40,
        "the catalog scan found only {} entries; the scanner is broken and the parity guard \
         would pass vacuously",
        entries.len()
    );
    entries
}

/// Resolve the receiver of a `.diagnostic(..)` call to the rule id it names.
///
/// Returns `None` for the one forwarding receiver inside `resolve_param`,
/// whose entry is supplied by its callers and counted there.
fn construction_rule_id<'a>(
    production: &str,
    catalog: &'a BTreeMap<String, String>,
    file: &Path,
    base: usize,
    site: usize,
) -> Option<&'a str> {
    let head = &production[..site];
    let name_start = head
        .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != ':')
        .map_or(0, |at| at + 1);
    let receiver = &head[name_start..];
    if receiver == FORWARDED_PARAM_NAME {
        return None;
    }
    let name = receiver
        .strip_prefix(CATALOG_PATH_PREFIX)
        .unwrap_or_else(|| {
            panic!(
                "{}: diagnostic construction at byte {} is built on `{receiver}` rather than a \
             rule catalog entry. Every diagnostic is constructed through its entry in \
             `src/{CATALOG_FILE}`, which is what keeps the validating side, the rule's \
             severity, and the rendered governance reference from drifting apart",
                file.display(),
                base + site
            )
        });
    Some(catalog.get(name).map_or_else(
        || {
            panic!(
                "{}: construction at byte {} references `catalog::{name}`, which is not a \
                 catalog entry carrying a rule id",
                file.display(),
                base + site
            )
        },
        String::as_str,
    ))
}

/// Resolve the `&catalog::NAME` entry a `resolve_param` call forwards.
fn forwarded_rule_id<'a>(
    production: &str,
    catalog: &'a BTreeMap<String, String>,
    file: &Path,
    base: usize,
    site: usize,
) -> &'a str {
    let rest = production[site..].trim_start();
    let name = rest.strip_prefix(FORWARDING_ARGUMENT_PREFIX).map_or_else(
        || {
            panic!(
                "{}: `{FORWARDING_MARKER}` call at byte {} does not forward a catalog entry \
                 (found `{rest_head}`); the forwarded rule would be invisible to this guard",
                file.display(),
                base + site,
                rest_head = leading_identifier(production, site)
            )
        },
        |tail| {
            let end = tail
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(tail.len());
            &tail[..end]
        },
    );
    catalog.get(name).map_or_else(
        || {
            panic!(
                "{}: `{FORWARDING_MARKER}` call at byte {} forwards `catalog::{name}`, which \
                 is not a catalog entry carrying a rule id",
                file.display(),
                base + site
            )
        },
        String::as_str,
    )
}

/// Reject every way of building a `Diagnostic` that bypasses the catalog.
///
/// `Diagnostic` and its fields are pinned public API (AFM-0026:R1, R7), so a
/// struct literal `Diagnostic { .. }` and a constructor bound as a value are
/// both legal Rust that would emit a rendered rule without consuming its
/// catalog entry — leaving this guard and the golden green while the
/// validating side silently stopped deriving anything. Scanning only for
/// `Diagnostic::warning(` missed both.
fn assert_no_direct_construction(production: &str, file: &Path, base: usize) {
    for constructor in DIAGNOSTIC_CONSTRUCTORS {
        if let Some(at) = production.find(constructor) {
            panic!(
                "{}: names the `{constructor}` constructor at byte {}. Diagnostics are built \
                 through `RuleEntry::diagnostic` in `src/{CATALOG_FILE}` and nowhere else, so \
                 that a rule's severity is decided by its catalog entry; naming the \
                 constructor here bypasses that even when it is not called on the spot",
                file.display(),
                base + at
            );
        }
    }
    let mut cursor = 0;
    while let Some(offset) = production[cursor..].find(DIAGNOSTIC_TYPE) {
        let at = cursor + offset;
        let after = at + DIAGNOSTIC_TYPE.len();
        cursor = after;
        let head = &production[..at];
        let is_whole_word = !head
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        if !is_whole_word || !production[after..].trim_start().starts_with('{') {
            continue;
        }
        let preceding = head
            .trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == ':')
            .trim_end();
        if TYPE_POSITION_TOKENS
            .iter()
            .any(|token| preceding.ends_with(token))
        {
            continue;
        }
        panic!(
            "{}: builds a `{DIAGNOSTIC_TYPE}` struct literal at byte {}. Its fields are \
             public, so this compiles and emits a real diagnostic while consuming no catalog \
             entry — the exact false-clean path this guard exists to close. Construct through \
             `RuleEntry::diagnostic` in `src/{CATALOG_FILE}`",
            file.display(),
            base + at
        );
    }
}

fn implemented_rule_ids() -> BTreeSet<String> {
    let catalog = catalog_entries();
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src_root, &mut files);
    files.sort();

    let mut ids = BTreeSet::new();
    let mut definitions = 0usize;
    let mut forwarding_receivers = 0usize;
    let mut forwarding_calls = 0usize;
    for file in &files {
        let is_canonical_construction = file.ends_with(CATALOG_FILE);
        let is_definition = file.ends_with(DEFINITION_FILE);
        if is_canonical_construction {
            continue;
        }
        let text = fs::read_to_string(file).expect("source file is readable");
        let is_definition_file = file.ends_with(FORWARDING_DEFINITION_FILE);
        for segment in production_segments(&text) {
            let production = segment.text;
            if forwarding_definition_span(production, file).is_some() {
                assert!(
                    is_definition_file,
                    "{}: `{FORWARDING_DEFINITION}` is defined outside the one canonical \
                     forwarding file `{FORWARDING_DEFINITION_FILE}`; this guard's exemption \
                     is bound to that definition and must not silently follow it elsewhere",
                    file.display()
                );
                definitions += 1;
            }
            if is_definition {
                assert!(
                    production.contains("pub struct Diagnostic {"),
                    "{}: is exempt from the direct-construction ban because it DEFINES \
                     `Diagnostic`; it no longer does, so the exemption is unbound and must \
                     be re-proven",
                    file.display()
                );
            } else {
                assert_no_direct_construction(production, file, segment.offset);
            }
            let mut cursor = 0;
            while let Some(offset) = production[cursor..].find(CONSTRUCTION_MARKER) {
                let site = cursor + offset;
                cursor = site + CONSTRUCTION_MARKER.len();
                match construction_rule_id(production, &catalog, file, segment.offset, site) {
                    Some(id) => {
                        ids.insert(id.to_string());
                    }
                    None => forwarding_receivers += 1,
                }
            }
            let mut cursor = 0;
            while let Some(offset) = production[cursor..].find(FORWARDING_MARKER) {
                let start = cursor + offset;
                let site = start + FORWARDING_MARKER.len();
                cursor = site;
                if production[..start].ends_with("fn ") {
                    continue;
                }
                forwarding_calls += 1;
                let comma = production[site..]
                    .find(',')
                    .map(|at| site + at + 1)
                    .expect("a resolve_param call names its rule after the config argument");
                ids.insert(
                    forwarded_rule_id(production, &catalog, file, segment.offset, comma)
                        .to_string(),
                );
            }
        }
    }

    assert_eq!(
        definitions, 1,
        "expected exactly one `{FORWARDING_DEFINITION}` definition in src/; found \
         {definitions}. This guard's only non-catalog receiver is bound to that single \
         definition"
    );
    assert_eq!(
        forwarding_receivers, 1,
        "expected exactly one forwarding `{FORWARDED_PARAM_NAME}.diagnostic(..)` receiver \
         (inside `{FORWARDING_DEFINITION}`); found {forwarding_receivers}. A changed count \
         means the exemption has generalised and must be re-proven"
    );
    assert!(
        forwarding_calls > 0,
        "no `{FORWARDING_MARKER}` call sites found; the forwarded rule ids would be invisible \
         to this guard"
    );
    assert!(
        ids.len() >= 40,
        "the construction-site scan found only {} rule ids; the scanner is broken and the \
         parity guard would pass vacuously",
        ids.len()
    );
    ids
}

fn governance_output() -> String {
    let dir = TempDir::new().expect("tempdir");
    fs::write(dir.path().join("adr-fmt.toml"), PARITY_CONFIG).expect("write config");
    fs::create_dir_all(dir.path().join("docs").join("adr").join("test"))
        .expect("create corpus directories");

    let output = Command::new(env!("CARGO_BIN_EXE_adr-fmt"))
        .current_dir(dir.path())
        .output()
        .expect("adr-fmt binary runs");
    assert!(
        output.status.success(),
        "default-mode governance output must exit 0, got {:?}",
        output.status
    );
    String::from_utf8(output.stdout).expect("governance output is utf-8")
}

fn registry_entries(stdout: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("    ") else {
            continue;
        };
        let Some((id, tail)) = rest.split_once(' ') else {
            continue;
        };
        if is_rule_id(id) && !tail.trim().is_empty() {
            entries.push((id.to_string(), tail.trim().to_string()));
        }
    }
    entries
}

fn rendered_rule_ids(stdout: &str) -> BTreeSet<String> {
    let ids: BTreeSet<String> = registry_entries(stdout)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert!(
        !ids.is_empty(),
        "no registry lines parsed from governance output; the parity guard would pass \
         vacuously"
    );
    ids
}

fn registry_description(stdout: &str, id: &str) -> String {
    registry_entries(stdout)
        .into_iter()
        .find(|(entry_id, _)| entry_id == id)
        .unwrap_or_else(|| panic!("governance output has no registry entry for `{id}`"))
        .1
}

#[test]
fn every_implemented_rule_is_rendered_in_governance_output() {
    let implemented = implemented_rule_ids();
    let rendered = rendered_rule_ids(&governance_output());
    let missing: Vec<&String> = implemented.difference(&rendered).collect();
    assert!(
        missing.is_empty(),
        "these rules are implemented in src/ but carry no described entry in the governance \
         reference, which claims to be the single source of truth for all invariant rules: \
         {missing:?}"
    );
}

#[test]
fn every_rendered_rule_is_implemented_in_src() {
    let implemented = implemented_rule_ids();
    let rendered = rendered_rule_ids(&governance_output());
    let bogus: Vec<&String> = rendered.difference(&implemented).collect();
    assert!(
        bogus.is_empty(),
        "the governance reference documents these rules, but no diagnostic construction site \
         in src/ emits them: {bogus:?}"
    );
}

#[test]
fn naming_registry_descriptions_match_afm_0008() {
    let stdout = governance_output();
    for (id, keywords, requirement) in [
        ("N001", &["kebab-slug"][..], "AFM-0008:R1 filename pattern"),
        ("N002", &["title"][..], "AFM-0008:R1 H1 title identifier"),
        (
            "N003",
            &["lowercase", "kebab"][..],
            "AFM-0008:R4 lowercase kebab-case slug",
        ),
        (
            "N004",
            &["prefix", "domain"][..],
            "AFM-0008:R2 unregistered prefix",
        ),
    ] {
        let description = registry_description(&stdout, id).to_lowercase();
        for keyword in keywords {
            assert!(
                description.contains(keyword),
                "{id}'s governance description must state {requirement} (missing `{keyword}`); \
                 got: {description}"
            );
        }
    }
}
