use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

mod common;
use common::rust_sources;

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

const GOLDEN_COUPLINGS: [&str; 1] = ["toml::Value @ config::RuleConfig::params"];

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

enum Owners {
    Unique(String),
    Ambiguous(BTreeSet<String>),
}

impl Owners {
    fn add(&mut self, module: &str) {
        match self {
            Self::Unique(first) if first == module => {}
            Self::Unique(first) => {
                *self = Self::Ambiguous([first.clone(), module.to_owned()].into());
            }
            Self::Ambiguous(all) => {
                all.insert(module.to_owned());
            }
        }
    }
}

struct Walk {
    modules: BTreeMap<String, Module>,
    owner_of: BTreeMap<String, Owners>,
    deps: BTreeSet<String>,
    couplings: BTreeSet<String>,
    reached: BTreeSet<String>,
}

impl Walk {
    fn load() -> Self {
        let deps = dependency_crates();
        let src = manifest_dir().join("src");
        let mut files = Vec::new();
        rust_sources(&src, &mut files).expect("src directory is readable");
        files.sort();

        let mut modules = BTreeMap::new();
        let mut owner_of: BTreeMap<String, Owners> = BTreeMap::new();
        for file in &files {
            let name = file
                .strip_prefix(&src)
                .expect("source lives under src")
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "::");
            let module = index_module(file, &deps);
            for item in module.items.keys() {
                owner_of
                    .entry(item.clone())
                    .and_modify(|owners| owners.add(&name))
                    .or_insert_with(|| Owners::Unique(name.clone()));
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
        let Some((last, prefix)) = segments.split_last() else {
            return Resolution::Ignored;
        };

        if prefix.is_empty() {
            if let Some(module) = self.site_module(site)
                && let Some(dep) = module.dep_bindings.get(last)
            {
                return Resolution::ThirdParty(format!("{dep}::{last} @ {site}"));
            }
            return self.unqualified(last, site);
        }

        let root = prefix[0].as_str();
        if self.deps.contains(root) {
            return Resolution::ThirdParty(format!("{root}::{last} @ {site}"));
        }
        if PATH_ROOTS_STD.contains(&root) {
            return Resolution::Ignored;
        }
        match self.module_key(prefix, last, site) {
            Some(module) => Resolution::Local(module, last.clone()),
            None => self.unqualified(last, site),
        }
    }

    fn module_key(&self, prefix: &[String], last: &str, site: &str) -> Option<String> {
        let mut rest = prefix;
        let mut segments: Vec<String> = Vec::new();
        while let Some(root) = rest.first().map(String::as_str)
            && PATH_ROOTS_LOCAL.contains(&root)
        {
            match root {
                "self" => segments = self.site_module_segments(site)?,
                "super" => {
                    if segments.is_empty() {
                        segments = self.site_module_segments(site)?;
                    }
                    segments.pop()?;
                }
                _ => segments.clear(),
            }
            rest = &rest[1..];
        }
        segments.extend_from_slice(rest);
        let joined = segments.join("::");
        [joined.clone(), format!("{joined}::mod")]
            .into_iter()
            .find(|candidate| {
                self.modules
                    .get(candidate)
                    .is_some_and(|module| module.items.contains_key(last))
            })
    }

    fn site_module_segments(&self, site: &str) -> Option<Vec<String>> {
        Some(
            self.site_module_key(site)?
                .split("::")
                .map(str::to_owned)
                .collect(),
        )
    }

    fn site_module_key(&self, site: &str) -> Option<String> {
        let segments: Vec<&str> = site.split("::").collect();
        (1..segments.len())
            .rev()
            .map(|cut| segments[..cut].join("::"))
            .find(|candidate| self.modules.contains_key(candidate))
    }

    fn site_module(&self, site: &str) -> Option<&Module> {
        self.modules.get(&self.site_module_key(site)?)
    }

    fn unqualified(&self, name: &str, site: &str) -> Resolution {
        match self.owner_of.get(name) {
            None => Resolution::Ignored,
            Some(Owners::Unique(module)) => Resolution::Local(module.clone(), name.to_owned()),
            Some(Owners::Ambiguous(modules)) => panic!(
                "{site}: the bare name `{name}` is defined in {modules:?}. This walk cannot \
                 tell which definition the path denotes, so it could follow the wrong one and \
                 miss that definition's third-party coupling. Spell the path with its module"
            ),
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
         ADR. The walk is syntactic: macro-generated items, cross-crate alias chains, \
         associated types and trait impls are outside what it can see"
    );
}

#[test]
fn duplicate_local_names_are_recorded_as_ambiguous() {
    let walk = Walk::load();
    let ambiguous: Vec<&String> = walk
        .owner_of
        .iter()
        .filter(|(_, owners)| matches!(owners, Owners::Ambiguous(_)))
        .map(|(name, _)| name)
        .collect();
    assert!(
        ambiguous.contains(&&"check".to_owned()),
        "`check` is defined in several rule modules, so the index must record it as \
         ambiguous. Recording one owner per name would let a bare path resolve to the \
         wrong definition and miss its third-party coupling. Ambiguous: {ambiguous:?}"
    );
}
