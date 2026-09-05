//! Bidirectional parity guard between the diagnostics adr-fmt implements
//! and the rule registry its governance output renders.
//!
//! The rendered side is read from the real binary's stdout, never from
//! source text: a source-scanning guard is satisfied by an id appearing in
//! a comment or a test string and so fails open.
//!
//! The implemented side no longer scans for string literals. Every
//! diagnostic construction site now takes its id from a `src/rules/catalog.rs`
//! entry (`catalog::T002.id`), so this guard resolves those references
//! through the catalog. That keeps the check honest in both directions: an
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

const DIAGNOSTIC_MARKERS: [&str; 2] = ["Diagnostic::warning(", "Diagnostic::error("];

const FORWARDING_MARKER: &str = "resolve_param(";

const FORWARDING_DEFINITION: &str = "fn resolve_param(";

const FORWARDING_DEFINITION_FILE: &str = "rules/template.rs";

const FORWARDED_PARAM_NAME: &str = "rule";

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

/// Resolve `catalog::NAME.id` at a construction site to the rule id it names.
fn catalog_rule_id<'a>(
    production: &str,
    catalog: &'a BTreeMap<String, String>,
    file: &Path,
    base: usize,
    site: usize,
) -> &'a str {
    let rest = production[site..].trim_start();
    assert!(
        rest.starts_with(CATALOG_PATH_PREFIX),
        "{}: diagnostic construction at byte {} does not take its id from the rule catalog \
         (found `{}`). Rule ids are stated once, in `src/{CATALOG_FILE}`, so that the \
         validating side and the rendered governance reference cannot drift; a literal here \
         reintroduces exactly that drift",
        file.display(),
        base + site,
        leading_identifier(production, site)
    );
    let name_start = site + production[site..].len() - rest.len() + CATALOG_PATH_PREFIX.len();
    let name = leading_identifier(production, name_start);
    catalog.get(name).map_or_else(
        || {
            panic!(
                "{}: construction at byte {} references `catalog::{name}`, which is not a \
                 catalog entry carrying a rule id",
                file.display(),
                base + site
            )
        },
        String::as_str,
    )
}

fn implemented_rule_ids() -> BTreeSet<String> {
    let catalog = catalog_entries();
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src_root, &mut files);
    files.sort();

    let mut ids = BTreeSet::new();
    let mut definitions = 0usize;
    let mut exempted_sites = 0usize;
    let mut forwarding_calls = 0usize;
    for file in &files {
        if file.ends_with(CATALOG_FILE) {
            continue;
        }
        let text = fs::read_to_string(file).expect("source file is readable");
        let is_definition_file = file.ends_with(FORWARDING_DEFINITION_FILE);
        for segment in production_segments(&text) {
            let production = segment.text;
            let definition = forwarding_definition_span(production, file);
            if definition.is_some() {
                assert!(
                    is_definition_file,
                    "{}: `{FORWARDING_DEFINITION}` is defined outside the one canonical \
                     forwarding file `{FORWARDING_DEFINITION_FILE}`; the parity guard's \
                     exemption is bound to that definition and must not silently follow it \
                     elsewhere",
                    file.display()
                );
                definitions += 1;
            }
            for marker in DIAGNOSTIC_MARKERS {
                let mut cursor = 0;
                while let Some(offset) = production[cursor..].find(marker) {
                    let site = cursor + offset + marker.len();
                    cursor = site;
                    let identifier = leading_identifier(production, site);
                    let inside_definition =
                        definition.is_some_and(|(start, end)| site >= start && site < end);
                    if inside_definition && identifier == FORWARDED_PARAM_NAME {
                        exempted_sites += 1;
                        continue;
                    }
                    ids.insert(
                        catalog_rule_id(production, &catalog, file, segment.offset, site)
                            .to_string(),
                    );
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
                    catalog_rule_id(production, &catalog, file, segment.offset, comma).to_string(),
                );
            }
        }
    }

    assert_eq!(
        definitions, 1,
        "expected exactly one `{FORWARDING_DEFINITION}` definition in src/; found \
         {definitions}. The parity guard's only non-catalog exemption is bound to that \
         single definition"
    );
    assert_eq!(
        exempted_sites, 1,
        "expected exactly one exempted non-catalog diagnostic construction (the forwarding \
         site inside `{FORWARDING_DEFINITION}`); found {exempted_sites}. A changed count \
         means the exemption has generalised and must be re-proven"
    );
    assert!(
        forwarding_calls > 0,
        "no `{FORWARDING_MARKER}` call sites found; the forwarded rule ids would be invisible \
         to the parity guard"
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
