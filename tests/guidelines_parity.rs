//! Bidirectional parity: directly constructed diagnostics versus registry IDs
//! rendered by the real binary, not source strings.
//!
//! The `syn` AST guard rejects `Diagnostic` literals, `warning`/`error`
//! references (called or not, whitespace-independent), and preceding single-hop
//! `use`/`type` aliases. It performs no type resolution or macro expansion.
//!
//! Known compiling bypasses: `<Diagnostic>::warning`, macro invocations,
//! and forward alias chains. Closing these requires semantic resolution or
//! private fields; AFM-0026:R3 requires a successor ADR for that migration.
//!
//! Trusted base: `report.rs` defines `Diagnostic`; `rules/catalog.rs` supplies
//! `RuleEntry::diagnostic`, the intended severity decision. Exemptions assert
//! those roles, not within-file uniqueness; internal bypasses remain trusted.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

mod common;
use common::rust_sources;
use std::process::Command;

use syn::visit::Visit;
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

const DIAGNOSTIC_TYPE: &str = "Diagnostic";

const CONSTRUCTORS: [&str; 2] = ["warning", "error"];

const CONSTRUCTION_METHOD: &str = "diagnostic";

const FORWARDING_FUNCTION: &str = "resolve_param";

const FORWARDED_RECEIVER: &str = "rule";

const CATALOG_MODULE: &str = "catalog";

const CATALOG_FILE: &str = "rules/catalog.rs";

const DEFINITION_FILE: &str = "report.rs";

const ENTRY_TYPE: &str = "RuleEntry";

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

fn parse(file: &Path) -> syn::File {
    let text = fs::read_to_string(file).expect("source file is readable");
    syn::parse_file(&text)
        .unwrap_or_else(|e| panic!("{}: source does not parse: {e}", file.display()))
}

fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => Some(path.path.segments.last()?.ident.to_string()),
        _ => None,
    }
}

/// The first string literal appearing anywhere in an expression.
struct FirstStringLiteral(Option<String>);

impl<'ast> Visit<'ast> for FirstStringLiteral {
    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        if self.0.is_none() {
            self.0 = Some(node.value());
        }
    }
}

/// Maps each catalog constant name to the rule id it carries.
///
/// Read from source rather than imported because `catalog` is crate-private
/// (AFM-0026:R2) and an integration test cannot see it. The constant name and
/// the id differ in case for `T005c`, so the mapping is read, never guessed.
struct CatalogEntries(BTreeMap<String, String>);

impl<'ast> Visit<'ast> for CatalogEntries {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        if type_name(&node.ty).as_deref() != Some(ENTRY_TYPE) {
            return;
        }
        let mut first = FirstStringLiteral(None);
        first.visit_expr(&node.expr);
        match first.0 {
            Some(id) if is_rule_id(&id) => {
                self.0.insert(node.ident.to_string(), id);
            }
            _ => panic!(
                "catalog entry `{}` does not open with a literal rule id",
                node.ident
            ),
        }
    }
}

fn catalog_entries() -> BTreeMap<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(CATALOG_FILE);
    let mut entries = CatalogEntries(BTreeMap::new());
    entries.visit_file(&parse(&path));
    assert!(
        entries.0.len() >= 40,
        "the catalog scan found only {} entries; the scanner is broken and this guard would \
         pass vacuously",
        entries.0.len()
    );
    entries.0
}

/// Local spellings that appear to denote `Diagnostic` in one file.
///
/// This is a one-pass token walk, NOT name resolution: it sees a `use ... as`
/// rename and a `type` alias whose right-hand side is already a known
/// spelling, and it sees them only if they are declared before the use site.
/// A forward chain, a macro-generated alias, or an alias introduced through
/// a re-export are all invisible to it.
struct LocalDiagnosticSpellings(BTreeSet<String>);

impl<'ast> Visit<'ast> for LocalDiagnosticSpellings {
    fn visit_use_tree(&mut self, node: &'ast syn::UseTree) {
        match node {
            syn::UseTree::Name(name) if name.ident == DIAGNOSTIC_TYPE => {
                self.0.insert(name.ident.to_string());
            }
            syn::UseTree::Rename(rename) if rename.ident == DIAGNOSTIC_TYPE => {
                self.0.insert(rename.rename.to_string());
            }
            _ => {}
        }
        syn::visit::visit_use_tree(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        if type_name(&node.ty).is_some_and(|name| self.0.contains(&name)) {
            self.0.insert(node.ident.to_string());
        }
        syn::visit::visit_item_type(self, node);
    }
}

fn local_diagnostic_spellings(file: &syn::File) -> BTreeSet<String> {
    let mut names = LocalDiagnosticSpellings(BTreeSet::new());
    names.0.insert(DIAGNOSTIC_TYPE.to_string());
    names.visit_file(file);
    names.0
}

struct Sites<'a> {
    catalog: &'a BTreeMap<String, String>,
    names: BTreeSet<String>,
    file: String,
    enforce_direct_construction_ban: bool,
    ids: BTreeSet<String>,
    forwarded_receivers: usize,
    forwarding_calls: usize,
}

impl Sites<'_> {
    fn rule_id(&self, name: &syn::Ident) -> String {
        let name = name.to_string();
        self.catalog.get(&name).cloned().unwrap_or_else(|| {
            panic!(
                "{}: references `{CATALOG_MODULE}::{name}`, which is not a catalog entry \
                 carrying a rule id",
                self.file
            )
        })
    }

    /// `catalog::NAME` as an expression, if that is what this path is.
    fn catalog_entry_name(expr: &syn::Expr) -> Option<syn::Ident> {
        let syn::Expr::Path(path) = strip_reference(expr) else {
            return None;
        };
        let mut segments = path.path.segments.iter().rev();
        let name = segments.next()?.ident.clone();
        let module = segments.next()?;
        (module.ident == CATALOG_MODULE).then_some(name)
    }
}

fn strip_reference(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Reference(reference) => strip_reference(&reference.expr),
        other => other,
    }
}

impl<'ast> Visit<'ast> for Sites<'_> {
    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if self.enforce_direct_construction_ban
            && node
                .path
                .segments
                .last()
                .is_some_and(|segment| self.names.contains(&segment.ident.to_string()))
        {
            panic!(
                "{}: builds a `{DIAGNOSTIC_TYPE}` struct literal. Its fields are public \
                 (AFM-0026:R1, R7), so this compiles and emits a real diagnostic while \
                 consuming no catalog entry — a false clean. Construct through \
                 `RuleEntry::diagnostic` in `src/{CATALOG_FILE}`. Note this check covers \
                 direct spellings only; see the module doc for the forms it cannot see",
                self.file
            );
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        let mut segments = node.path.segments.iter().rev();
        if let (Some(last), Some(owner)) = (segments.next(), segments.next())
            && self.enforce_direct_construction_ban
            && CONSTRUCTORS.contains(&last.ident.to_string().as_str())
            && self.names.contains(&owner.ident.to_string())
        {
            panic!(
                "{}: names the `{DIAGNOSTIC_TYPE}::{}` constructor. Diagnostics are \
                 INTENDED to be built through `RuleEntry::diagnostic` in `src/{CATALOG_FILE}` \
                 so that a rule's severity is decided by its catalog entry; naming the \
                 constructor bypasses that even when it is not called here",
                self.file, last.ident
            );
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == CONSTRUCTION_METHOD {
            match Sites::catalog_entry_name(&node.receiver) {
                Some(name) => {
                    self.ids.insert(self.rule_id(&name));
                }
                None => match strip_reference(&node.receiver) {
                    syn::Expr::Path(path) if path.path.is_ident(FORWARDED_RECEIVER) => {
                        self.forwarded_receivers += 1;
                    }
                    _ => panic!(
                        "{}: calls `.{CONSTRUCTION_METHOD}(..)` on something that is neither a \
                         `{CATALOG_MODULE}::` entry nor the one forwarding receiver; its rule \
                         id would be invisible to this guard",
                        self.file
                    ),
                },
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref()
            && path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == FORWARDING_FUNCTION)
        {
            self.forwarding_calls += 1;
            let entry = node
                .args
                .iter()
                .find_map(Sites::catalog_entry_name)
                .unwrap_or_else(|| {
                    panic!(
                        "{}: a `{FORWARDING_FUNCTION}` call forwards no `{CATALOG_MODULE}::` \
                         entry, so the forwarded rule would be invisible to this guard",
                        self.file
                    )
                });
            self.ids.insert(self.rule_id(&entry));
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn directly_constructed_rule_ids() -> BTreeSet<String> {
    let catalog = catalog_entries();
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src_root, &mut files).expect("src directory is readable");
    files.sort();

    let mut ids = BTreeSet::new();
    let mut forwarded_receivers = 0usize;
    let mut forwarding_calls = 0usize;
    let mut definition_files = 0usize;
    let mut canonical_files = 0usize;

    for file in &files {
        let ast = parse(file);
        let is_definition = file.ends_with(DEFINITION_FILE);
        let is_canonical = file.ends_with(CATALOG_FILE);

        if is_definition {
            definition_files += 1;
            assert!(
                defines_diagnostic(&ast),
                "{}: is exempt from the construction ban because it DEFINES \
                 `{DIAGNOSTIC_TYPE}`; it no longer does, so the exemption is unbound and must \
                 be re-proven",
                file.display()
            );
        }
        if is_canonical {
            canonical_files += 1;
            assert!(
                defines_construction(&ast),
                "{}: is exempt because it holds the crate's only construction site \
                 (`RuleEntry::{CONSTRUCTION_METHOD}`); it no longer does, so the exemption is \
                 unbound",
                file.display()
            );
        }

        let mut sites = Sites {
            catalog: &catalog,
            names: local_diagnostic_spellings(&ast),
            file: file.display().to_string(),
            enforce_direct_construction_ban: !is_definition && !is_canonical,
            ids: BTreeSet::new(),
            forwarded_receivers: 0,
            forwarding_calls: 0,
        };
        sites.visit_file(&ast);
        ids.extend(sites.ids);
        forwarded_receivers += sites.forwarded_receivers;
        forwarding_calls += sites.forwarding_calls;
    }

    assert_eq!(
        definition_files, 1,
        "expected exactly one file defining `{DIAGNOSTIC_TYPE}`; found {definition_files}"
    );
    assert_eq!(
        canonical_files, 1,
        "expected exactly one canonical construction file; found {canonical_files}"
    );
    assert_eq!(
        forwarded_receivers, 1,
        "expected exactly one forwarding `{FORWARDED_RECEIVER}.{CONSTRUCTION_METHOD}(..)` \
         receiver; found {forwarded_receivers}. A changed count means the exemption has \
         generalised and must be re-proven"
    );
    assert!(
        forwarding_calls > 0,
        "no `{FORWARDING_FUNCTION}` call sites found; the forwarded rule ids would be \
         invisible to this guard"
    );
    assert!(
        ids.len() >= 40,
        "the direct-construction walk found only {} rule ids; the walker is broken and this \
         guard would pass vacuously",
        ids.len()
    );
    ids
}

fn defines_diagnostic(ast: &syn::File) -> bool {
    ast.items.iter().any(|item| match item {
        syn::Item::Struct(item) => item.ident == DIAGNOSTIC_TYPE,
        _ => false,
    })
}

fn defines_construction(ast: &syn::File) -> bool {
    ast.items.iter().any(|item| match item {
        syn::Item::Impl(item) => {
            type_name(&item.self_ty).as_deref() == Some(ENTRY_TYPE)
                && item.items.iter().any(|member| match member {
                    syn::ImplItem::Fn(function) => function.sig.ident == CONSTRUCTION_METHOD,
                    _ => false,
                })
        }
        _ => false,
    })
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
fn every_directly_constructed_rule_is_rendered_in_governance_output() {
    let constructed = directly_constructed_rule_ids();
    let rendered = rendered_rule_ids(&governance_output());
    let missing: Vec<&String> = constructed.difference(&rendered).collect();
    assert!(
        missing.is_empty(),
        "these rules have a direct construction site in src/ but carry no described entry \
         in the governance reference, which claims to be the single source of truth for all \
         invariant rules: {missing:?}"
    );
}

#[test]
fn every_rendered_rule_has_a_direct_construction_site() {
    let constructed = directly_constructed_rule_ids();
    let rendered = rendered_rule_ids(&governance_output());
    let bogus: Vec<&String> = rendered.difference(&constructed).collect();
    assert!(
        bogus.is_empty(),
        "the governance reference documents these rules, but no DIRECT diagnostic \
         construction site in src/ emits them: {bogus:?}"
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

#[test]
fn generated_authority_adopts_six_techniques_without_semantic_guarantees() {
    let stdout = governance_output();
    for required in [
        "AUTHOR AND REVIEW OBLIGATIONS",
        "standalone action, object and scope",
        "Read the exact cited rule",
        "--context example-core",
        "+ becomes easier:",
        "Evidence freshness and entailment",
        "Acceptance and supersession",
        "Deterministic checks",
        "--max-warning-docs 0",
        "not a clean verdict",
    ] {
        assert!(
            stdout.contains(required),
            "missing adopted guidance: {required}"
        );
    }
    assert!(
        !stdout.contains("Example: Root: CHE-0001 | References:")
            && !stdout.contains("exploiting LLM primacy bias"),
        "examples must obey relationships and avoid unmeasured efficacy claims"
    );
}

#[test]
fn generated_examples_lint_and_retrieve_as_a_complete_corpus() {
    let dir = TempDir::new().expect("tempdir");
    let config = PARITY_CONFIG
        .replace("TST", "CHE")
        .replace("crates = []", "crates = [\"example-core\"]");
    fs::write(dir.path().join("adr-fmt.toml"), config).expect("write config");
    let corpus = dir.path().join("docs/adr/test");
    fs::create_dir_all(&corpus).expect("create corpus");
    let governance = governance_output();
    let empty = TempDir::new().expect("empty workspace");
    let setup = Command::new(env!("CARGO_BIN_EXE_adr-fmt"))
        .current_dir(empty.path())
        .output()
        .expect("render setup");
    assert!(setup.status.success(), "setup must render: {setup:?}");
    let setup = String::from_utf8(setup.stdout).expect("utf8");
    for rendered in [&governance, &setup] {
        let examples = rendered
            .split_once("  BEGIN EXAMPLES\n")
            .expect("generated complete examples start")
            .1
            .split_once("  END EXAMPLES")
            .expect("generated complete examples end")
            .0;
        let docs = examples
            .lines()
            .map(|line| line.strip_prefix("     ").unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n");
        let (root, child) = docs.split_once("# CHE-0002.").expect("two example ADRs");
        fs::write(corpus.join("CHE-0001-visible-input.md"), root).expect("write root");
        fs::write(
            corpus.join("CHE-0002-result-format.md"),
            format!("# CHE-0002.{child}"),
        )
        .expect("write child");
        let lint = Command::new(env!("CARGO_BIN_EXE_adr-fmt"))
            .current_dir(dir.path())
            .arg("--lint")
            .output()
            .expect("lint examples");
        assert!(lint.status.success(), "example lint must run: {lint:?}");
        assert_eq!(
            String::from_utf8(lint.stdout).expect("utf8").trim(),
            "## Diagnostics: 0 warning(s) across 2 ADR(s)",
            "generated examples must satisfy real grammar and budgets"
        );
        let context = Command::new(env!("CARGO_BIN_EXE_adr-fmt"))
            .current_dir(dir.path())
            .args(["--context", "example-core"])
            .output()
            .expect("retrieve examples");
        assert!(
            context.status.success(),
            "example retrieval must run: {context:?}"
        );
        let text = String::from_utf8(context.stdout).expect("utf8");
        for id in ["CHE-0001:R1:L5", "CHE-0002:R1:L5"] {
            assert!(text.contains(id), "example retrieval omitted {id}");
        }
    }
}
