//! Structural enumeration of third-party types reachable from the
//! AFM-0026:R1 public set, replacing the grep sweep that R9's coupling
//! census previously rested on.
//!
//! # What this guard enforces
//!
//! Starting from the R1 re-export set as it is spelled in `src/lib.rs`,
//! the walk follows public type-bearing syntax transitively — public
//! struct fields, every enum variant field, `pub fn` signatures in
//! inherent impls, free `pub fn` parameters and returns, type aliases,
//! generic bounds and `impl Trait` bounds — through locally defined
//! types, in the sense AFM-0026:R7 gives to "reachable". Every type or
//! trait spelling along the way that belongs to a `[dependencies]` crate
//! is recorded with the site that reaches it, and the resulting set must
//! equal `GOLDEN_COUPLINGS` exactly.
//!
//! This is the check a grep cannot perform. `config::RuleConfig` is NOT
//! in the R1 set; it is reached only through `Config::rules`, so a text
//! search over the R1 names never visits the one coupling that exists.
//!
//! # What this guard does NOT enforce
//!
//! `syn` parses tokens. It does not resolve names, does not expand
//! macros, and does not consult rustc. This is pattern-matching over
//! spellings, so the honest claim is narrower than R9's prohibition:
//!
//! - A third-party type reached through a macro-generated item, or named
//!   only inside a macro invocation, is invisible.
//! - A type alias chain that leaves this crate — `pub use dep::T as U`
//!   re-exported from elsewhere and then aliased — is not followed.
//! - Associated types (`<T as Trait>::Assoc`) and blanket generic
//!   parameters are not resolved to concrete types.
//! - Trait impls are not inspected. `#[derive(Deserialize)]` on a local
//!   type is a foreign trait implemented for a local type, which R9
//!   exempts, and this walk never reads attributes.
//! - Items public at the crate root but outside the R1 set — today only
//!   `run` — are not walked, because R9 scopes itself to the R1 set.
//!
//! Closing that class needs semantic resolution over a compiled crate.
//! The claim R9 makes is therefore the one this walk can back: the
//! couplings its syntactic walk reaches, not the couplings that exist.
//!
//! Vacuity is guarded separately: the extracted R1 set must match the
//! set R1 pins, the walk must reach a floor of local types including the
//! four that carry the surface, and a glob import from a dependency
//! anywhere in `src/` is rejected outright because it would let a bare
//! identifier denote a third-party type with no binding to read.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

/// The AFM-0026:R1 pinned re-export set, as `(module, item)`.
const R1_SET: [(&str, &str); 23] = [
    ("config", "Config"),
    ("config", "LoadError"),
    ("config", "ResolveCorpusError"),
    ("config", "load_quiet"),
    ("config", "resolve_corpus_root"),
    ("containment", "ContainmentError"),
    ("containment", "contained_join"),
    ("containment", "contained_join_optional"),
    ("model", "AdrId"),
    ("model", "AdrIdError"),
    ("model", "AdrRecord"),
    ("model", "DomainDir"),
    ("model", "RelVerb"),
    ("model", "Relationship"),
    ("model", "Status"),
    ("model", "Tier"),
    ("model", "parse_adr_id"),
    ("parser", "ParseError"),
    ("parser", "ParseOutcome"),
    ("parser", "parse_domain"),
    ("parser", "parse_stale"),
    ("report", "Diagnostic"),
    ("report", "Severity"),
];

/// Every third-party coupling the walk reaches, as `spelling @ site`.
const GOLDEN_COUPLINGS: [&str; 1] = ["toml::Value @ config::RuleConfig::params"];

/// Local types the walk must reach, or it is not exercising R7 transitivity.
const REQUIRED_REACH: [&str; 4] = ["Config", "RuleConfig", "AdrRecord", "Diagnostic"];

const REACH_FLOOR: usize = 15;

const PATH_ROOTS_LOCAL: [&str; 3] = ["crate", "self", "super"];

const PATH_ROOTS_STD: [&str; 3] = ["std", "core", "alloc"];

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn dependency_crates() -> BTreeSet<String> {
    let text = fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("Cargo.toml readable");
    let mut deps = BTreeSet::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[dependencies]";
            continue;
        }
        if !inside {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            let name = name.trim();
            if !name.is_empty() && !name.starts_with('#') {
                deps.insert(name.replace('-', "_"));
            }
        }
    }
    assert!(
        deps.len() >= 2,
        "read only {} entries from [dependencies]; the manifest scan is broken and this \
         guard would pass vacuously",
        deps.len()
    );
    deps
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

fn parse(file: &Path) -> syn::File {
    let text = fs::read_to_string(file).expect("source file is readable");
    syn::parse_file(&text)
        .unwrap_or_else(|e| panic!("{}: source does not parse: {e}", file.display()))
}

fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn self_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => Some(path.path.segments.last()?.ident.to_string()),
        _ => None,
    }
}

/// One source file, indexed by what the walk needs to read from it.
struct Module {
    items: BTreeMap<String, syn::Item>,
    inherent: BTreeMap<String, Vec<syn::Signature>>,
    dep_bindings: BTreeMap<String, String>,
}

fn collect_dep_bindings(
    tree: &syn::UseTree,
    root: Option<&str>,
    deps: &BTreeSet<String>,
    file: &str,
    out: &mut BTreeMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let next = root.map_or_else(|| path.ident.to_string(), str::to_owned);
            collect_dep_bindings(&path.tree, Some(&next), deps, file, out);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_dep_bindings(item, root, deps, file, out);
            }
        }
        syn::UseTree::Name(name) => {
            if let Some(root) = root
                && deps.contains(root)
            {
                out.insert(name.ident.to_string(), root.to_owned());
            }
        }
        syn::UseTree::Rename(rename) => {
            if let Some(root) = root
                && deps.contains(root)
            {
                out.insert(rename.rename.to_string(), root.to_owned());
            }
        }
        syn::UseTree::Glob(_) => {
            assert!(
                !root.is_some_and(|root| deps.contains(root)),
                "{file}: glob-imports from a dependency crate. A bare identifier could then \
                 denote a third-party type with no binding for this guard to read, so the \
                 coupling census would fail clean"
            );
        }
    }
}

fn index_module(path: &Path, deps: &BTreeSet<String>) -> Module {
    let ast = parse(path);
    let file = path.display().to_string();
    let mut items = BTreeMap::new();
    let mut inherent: BTreeMap<String, Vec<syn::Signature>> = BTreeMap::new();
    let mut dep_bindings = BTreeMap::new();

    for item in ast.items {
        match &item {
            syn::Item::Use(node) => {
                collect_dep_bindings(&node.tree, None, deps, &file, &mut dep_bindings);
            }
            syn::Item::Impl(node) if node.trait_.is_none() => {
                if let Some(owner) = self_type_name(&node.self_ty) {
                    let sigs = inherent.entry(owner).or_default();
                    for member in &node.items {
                        if let syn::ImplItem::Fn(function) = member
                            && is_public(&function.vis)
                        {
                            sigs.push(function.sig.clone());
                        }
                    }
                }
            }
            syn::Item::Struct(node) => {
                items.insert(node.ident.to_string(), item.clone());
            }
            syn::Item::Enum(node) => {
                items.insert(node.ident.to_string(), item.clone());
            }
            syn::Item::Fn(node) => {
                items.insert(node.sig.ident.to_string(), item.clone());
            }
            syn::Item::Type(node) => {
                items.insert(node.ident.to_string(), item.clone());
            }
            _ => {}
        }
    }

    Module {
        items,
        inherent,
        dep_bindings,
    }
}

/// A type or trait spelling encountered in a public position.
struct Spelling {
    path: syn::Path,
    site: String,
}

fn push_type(ty: &syn::Type, site: &str, out: &mut Vec<Spelling>) {
    match ty {
        syn::Type::Path(node) => {
            if let Some(qself) = &node.qself {
                push_type(&qself.ty, site, out);
            }
            out.push(Spelling {
                path: node.path.clone(),
                site: site.to_owned(),
            });
            for segment in &node.path.segments {
                push_path_arguments(&segment.arguments, site, out);
            }
        }
        syn::Type::Reference(node) => push_type(&node.elem, site, out),
        syn::Type::Slice(node) => push_type(&node.elem, site, out),
        syn::Type::Array(node) => push_type(&node.elem, site, out),
        syn::Type::Ptr(node) => push_type(&node.elem, site, out),
        syn::Type::Paren(node) => push_type(&node.elem, site, out),
        syn::Type::Group(node) => push_type(&node.elem, site, out),
        syn::Type::Tuple(node) => {
            for elem in &node.elems {
                push_type(elem, site, out);
            }
        }
        syn::Type::ImplTrait(node) => push_bounds(&node.bounds, site, out),
        syn::Type::TraitObject(node) => push_bounds(&node.bounds, site, out),
        syn::Type::FnPtr(node) => {
            for input in &node.inputs {
                push_type(&input.ty, site, out);
            }
            push_return(&node.output, site, out);
        }
        _ => {}
    }
}

fn push_path_arguments(arguments: &syn::PathArguments, site: &str, out: &mut Vec<Spelling>) {
    match arguments {
        syn::PathArguments::AngleBracketed(args) => {
            for arg in &args.args {
                match arg {
                    syn::GenericArgument::Type(ty) => push_type(ty, site, out),
                    syn::GenericArgument::AssocType(assoc) => push_type(&assoc.ty, site, out),
                    syn::GenericArgument::Constraint(constraint) => {
                        push_bounds(&constraint.bounds, site, out);
                    }
                    _ => {}
                }
            }
        }
        syn::PathArguments::Parenthesized(args) => {
            for input in &args.inputs {
                push_type(&input.ty, site, out);
            }
            push_return(&args.output, site, out);
        }
        syn::PathArguments::None => {}
    }
}

fn push_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    site: &str,
    out: &mut Vec<Spelling>,
) {
    for bound in bounds {
        if let syn::TypeParamBound::Trait(bound) = bound {
            out.push(Spelling {
                path: bound.path.clone(),
                site: site.to_owned(),
            });
            for segment in &bound.path.segments {
                push_path_arguments(&segment.arguments, site, out);
            }
        }
    }
}

fn push_return(output: &syn::ReturnType, site: &str, out: &mut Vec<Spelling>) {
    if let syn::ReturnType::Type(_, ty) = output {
        push_type(ty, site, out);
    }
}

fn push_generics(generics: &syn::Generics, site: &str, out: &mut Vec<Spelling>) {
    for param in &generics.params {
        if let syn::GenericParam::Type(param) = param {
            push_bounds(&param.bounds, site, out);
        }
    }
    if let Some(clause) = &generics.where_clause {
        for predicate in &clause.predicates {
            if let syn::WherePredicate::Type(predicate) = predicate {
                push_bounds(&predicate.bounds, site, out);
            }
        }
    }
}

fn push_signature(sig: &syn::Signature, site: &str, out: &mut Vec<Spelling>) {
    for input in &sig.inputs {
        if let syn::FnArg::Typed(arg) = input {
            push_type(&arg.ty, site, out);
        }
    }
    push_return(&sig.output, site, out);
    push_generics(&sig.generics, site, out);
}

fn push_fields(fields: &syn::Fields, owner: &str, all_public: bool, out: &mut Vec<Spelling>) {
    for (position, field) in fields.iter().enumerate() {
        if !all_public && !is_public(&field.vis) {
            continue;
        }
        let name = field
            .ident
            .as_ref()
            .map_or_else(|| position.to_string(), ToString::to_string);
        push_type(&field.ty, &format!("{owner}::{name}"), out);
    }
}

fn spellings_of(
    module: &str,
    name: &str,
    item: &syn::Item,
    inherent: &[syn::Signature],
) -> Vec<Spelling> {
    let owner = format!("{module}::{name}");
    let mut out = Vec::new();
    match item {
        syn::Item::Struct(node) => {
            push_fields(&node.fields, &owner, false, &mut out);
            push_generics(&node.generics, &owner, &mut out);
        }
        syn::Item::Enum(node) => {
            for variant in &node.variants {
                push_fields(
                    &variant.fields,
                    &format!("{owner}::{}", variant.ident),
                    true,
                    &mut out,
                );
            }
            push_generics(&node.generics, &owner, &mut out);
        }
        syn::Item::Fn(node) => push_signature(&node.sig, &owner, &mut out),
        syn::Item::Type(node) => {
            push_type(&node.ty, &owner, &mut out);
            push_generics(&node.generics, &owner, &mut out);
        }
        _ => {}
    }
    for sig in inherent {
        push_signature(sig, &format!("{owner}::{}", sig.ident), &mut out);
    }
    out
}

struct Walk {
    modules: BTreeMap<String, Module>,
    owner_of: BTreeMap<String, String>,
    deps: BTreeSet<String>,
    couplings: BTreeSet<String>,
    reached: BTreeSet<String>,
}

impl Walk {
    fn load() -> Self {
        let deps = dependency_crates();
        let src = manifest_dir().join("src");
        let mut files = Vec::new();
        rust_sources(&src, &mut files);
        files.sort();

        let mut modules = BTreeMap::new();
        let mut owner_of: BTreeMap<String, String> = BTreeMap::new();
        for file in &files {
            let name = file
                .strip_prefix(&src)
                .expect("source lives under src")
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "::");
            let module = index_module(file, &deps);
            for item in module.items.keys() {
                owner_of.entry(item.clone()).or_insert_with(|| name.clone());
            }
            modules.insert(name, module);
        }

        Self {
            modules,
            owner_of,
            deps,
            couplings: BTreeSet::new(),
            reached: BTreeSet::new(),
        }
    }

    /// The R1 set as `src/lib.rs` actually spells it.
    fn declared_r1_set() -> BTreeSet<(String, String)> {
        let ast = parse(&manifest_dir().join("src").join("lib.rs"));
        let mut out = BTreeSet::new();
        for item in &ast.items {
            if let syn::Item::Use(node) = item
                && is_public(&node.vis)
            {
                collect_reexports(&node.tree, None, &mut out);
            }
        }
        out
    }

    fn resolve(&self, path: &syn::Path, site: &str) -> Resolution {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let Some(last) = segments.last() else {
            return Resolution::Ignored;
        };
        let root = &segments[0];

        if segments.len() > 1 {
            if self.deps.contains(root) {
                return Resolution::ThirdParty(format!("{root}::{last} @ {site}"));
            }
            if PATH_ROOTS_STD.contains(&root.as_str()) {
                return Resolution::Ignored;
            }
            if PATH_ROOTS_LOCAL.contains(&root.as_str()) {
                return self.local(last);
            }
            return self.local(last);
        }

        if let Some(module) = self
            .modules
            .get(site.split("::").next().unwrap_or_default())
            && let Some(dep) = module.dep_bindings.get(last)
        {
            return Resolution::ThirdParty(format!("{dep}::{last} @ {site}"));
        }
        self.local(last)
    }

    fn local(&self, name: &str) -> Resolution {
        match self.owner_of.get(name) {
            Some(module) => Resolution::Local(module.clone(), name.to_owned()),
            None => Resolution::Ignored,
        }
    }

    fn run(&mut self) {
        let mut queue: VecDeque<(String, String)> = R1_SET
            .iter()
            .map(|(module, item)| ((*module).to_owned(), (*item).to_owned()))
            .collect();
        let mut seen: BTreeSet<(String, String)> = queue.iter().cloned().collect();

        while let Some((module_name, item_name)) = queue.pop_front() {
            let module = self
                .modules
                .get(&module_name)
                .unwrap_or_else(|| panic!("no module `{module_name}` under src/"));
            let item = module.items.get(&item_name).unwrap_or_else(|| {
                panic!("`{module_name}::{item_name}` is not defined in src/{module_name}.rs")
            });
            let inherent = module.inherent.get(&item_name).cloned().unwrap_or_default();
            self.reached.insert(item_name.clone());

            for spelling in spellings_of(&module_name, &item_name, item, &inherent) {
                match self.resolve(&spelling.path, &spelling.site) {
                    Resolution::ThirdParty(record) => {
                        self.couplings.insert(record);
                    }
                    Resolution::Local(owner, name) => {
                        let key = (owner, name);
                        if seen.insert(key.clone()) {
                            queue.push_back(key);
                        }
                    }
                    Resolution::Ignored => {}
                }
            }
        }
    }
}

enum Resolution {
    ThirdParty(String),
    Local(String, String),
    Ignored,
}

fn collect_reexports(
    tree: &syn::UseTree,
    root: Option<&str>,
    out: &mut BTreeSet<(String, String)>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let next = root.map_or_else(|| path.ident.to_string(), str::to_owned);
            collect_reexports(&path.tree, Some(&next), out);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_reexports(item, root, out);
            }
        }
        syn::UseTree::Name(name) => {
            if let Some(root) = root {
                out.insert((root.to_owned(), name.ident.to_string()));
            }
        }
        syn::UseTree::Rename(rename) => {
            if let Some(root) = root {
                out.insert((root.to_owned(), rename.rename.to_string()));
            }
        }
        syn::UseTree::Glob(_) => panic!("src/lib.rs glob-re-exports; R1 pins an exact set"),
    }
}

#[test]
fn lib_rs_reexports_exactly_the_r1_set() {
    let declared = Walk::declared_r1_set();
    let pinned: BTreeSet<(String, String)> = R1_SET
        .iter()
        .map(|(module, item)| ((*module).to_owned(), (*item).to_owned()))
        .collect();
    assert_eq!(
        declared, pinned,
        "src/lib.rs no longer re-exports the AFM-0026:R1 set this guard walks. Until the \
         two agree the coupling census below is walking the wrong surface"
    );
}

#[test]
fn third_party_couplings_reachable_from_r1_match_the_census() {
    let mut walk = Walk::load();
    walk.run();

    for required in REQUIRED_REACH {
        assert!(
            walk.reached.contains(required),
            "the transitive walk never reached `{required}`, so it is not exercising \
             AFM-0026:R7 reachability and this guard would pass vacuously. Reached: {:?}",
            walk.reached
        );
    }
    assert!(
        walk.reached.len() >= REACH_FLOOR,
        "the transitive walk reached only {} local types; the walker is broken and this \
         guard would pass vacuously",
        walk.reached.len()
    );

    let golden: BTreeSet<String> = GOLDEN_COUPLINGS.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(
        walk.couplings, golden,
        "the set of third-party type and trait spellings reachable from the AFM-0026:R1 set \
         through public signatures and R7 field shape has changed. AFM-0026:R9 pins this \
         census; widening it couples this crate's semver to a dependency's and requires an \
         ADR. Note the walk is syntactic — see this file's module doc for the forms it \
         cannot see"
    );
}
