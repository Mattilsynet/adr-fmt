//! Bidirectional parity guard between the diagnostics adr-fmt implements
//! and the rule registry its governance output renders.
//!
//! The rendered side is read from the real binary's stdout, never from
//! source text: a source-scanning guard is satisfied by an id appearing in
//! a comment or a test string and so fails open.

use std::collections::BTreeSet;
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

const FORWARDING_MARKERS: [&str; 1] = ["resolve_param("];

const FORWARDED_PARAM_NAMES: [&str; 1] = ["rule"];

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

fn production_prefix(src: &str) -> &str {
    let mut cursor = 0;
    while let Some(offset) = src[cursor..].find("\n#[cfg(test)]\n") {
        let attribute = cursor + offset + 1;
        let body = attribute + "#[cfg(test)]\n".len();
        if src[body..].starts_with("mod ") {
            return &src[..attribute];
        }
        cursor = attribute + 1;
    }
    src
}

fn leading_identifier(production: &str, site: usize) -> &str {
    let rest = production[site..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    &rest[..end]
}

fn literal_rule_id<'a>(production: &'a str, file: &Path, site: usize) -> &'a str {
    let Some(open) = production[site..].find('"').map(|at| site + at + 1) else {
        panic!(
            "{}: diagnostic construction at byte {site} carries no literal rule id, so the \
             parity guard cannot see this site",
            file.display()
        );
    };
    let Some(close) = production[open..].find('"').map(|at| open + at) else {
        panic!(
            "{}: unterminated rule id literal at byte {open}",
            file.display()
        );
    };
    let id = &production[open..close];
    assert!(
        is_rule_id(id),
        "{}: diagnostic construction at byte {site} does not open with a literal rule id \
         (found `{id}`), so the parity guard cannot see this site",
        file.display()
    );
    id
}

fn implemented_rule_ids() -> BTreeSet<String> {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src_root, &mut files);
    files.sort();

    let mut ids = BTreeSet::new();
    for file in &files {
        let text = fs::read_to_string(file).expect("source file is readable");
        let production = production_prefix(&text);
        for marker in DIAGNOSTIC_MARKERS {
            let mut cursor = 0;
            while let Some(offset) = production[cursor..].find(marker) {
                let site = cursor + offset + marker.len();
                cursor = site;
                if FORWARDED_PARAM_NAMES.contains(&leading_identifier(production, site)) {
                    continue;
                }
                ids.insert(literal_rule_id(production, file, site).to_string());
            }
        }
        for marker in FORWARDING_MARKERS {
            let mut cursor = 0;
            while let Some(offset) = production[cursor..].find(marker) {
                let start = cursor + offset;
                let site = start + marker.len();
                cursor = site;
                if production[..start].ends_with("fn ") {
                    continue;
                }
                ids.insert(literal_rule_id(production, file, site).to_string());
            }
        }
    }

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
