//! Prohibition on `#[non_exhaustive]` public error enums in `src/`
//! (item-level `pub`; RST-0006, recorded by AFM-0041 amending AFM-0026:R6).
//!
//! AFM-0037:R3: this guard reaches error types only via the derive path and
//! the Error-trait suffix heuristic; it errs toward flagging without name
//! resolution. False cleans: a trait alias unrelated to `Error`, a non-`Path`
//! self type, an aliased self type, and a non-error `#[non_exhaustive]`
//! state/outcome enum (not mechanically covered).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

mod common;

use common::{manifest_dir, rust_sources};

const FILES_FLOOR: usize = 19;
const PUB_ENUM_FLOOR: usize = 14;
const MANUAL_IMPL_FLOOR: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrorShape {
    Derived,
    Manual,
}

impl ErrorShape {
    fn as_str(self) -> &'static str {
        match self {
            Self::Derived => "derive",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug)]
struct Occurrence {
    path: String,
    line: usize,
    enum_name: String,
    shape: ErrorShape,
}

impl Occurrence {
    fn describe(&self) -> String {
        let path = self.path.as_str();
        let line = self.line;
        let name = self.enum_name.as_str();
        let shape = self.shape.as_str();
        format!("{path}:{line} {name} ({shape})")
    }
}

struct Scan {
    files: usize,
    pub_enums: usize,
    all_enums: usize,
    manual_impls: BTreeSet<String>,
    occurrences: Vec<Occurrence>,
}

struct SourceFile {
    relative: String,
    text: String,
    ast: syn::File,
}

#[test]
fn no_public_non_exhaustive_error_enum() {
    let scan = scan_live();
    let files = scan.files;
    let pub_enums = scan.pub_enums;
    let all_enums = scan.all_enums;
    let manual_impls = scan.manual_impls.len();
    println!(
        "closed-error prohibition summary: {files} files, pub enums: {pub_enums}, all enums: \
         {all_enums}, {manual_impls} manual Error impls; scope: `pub enum` error types declared \
         in `src/` (item-level `pub`; includes two types that are not re-exported)"
    );
    assert!(
        files >= FILES_FLOOR,
        "the scan parsed only {files} Rust sources under src/; the walker is broken and this \
         prohibition would pass vacuously"
    );
    assert!(
        pub_enums >= PUB_ENUM_FLOOR,
        "the scan collected only {pub_enums} public enums under src/; the collector is broken \
         and this prohibition would pass vacuously"
    );
    assert!(
        manual_impls >= MANUAL_IMPL_FLOOR,
        "the scan found only {manual_impls} manual `std::error::Error`/`core::error::Error` \
         implementations; without them the manual shape would be invisible and this prohibition \
         would pass vacuously"
    );
    let violations = scan
        .occurrences
        .iter()
        .map(Occurrence::describe)
        .collect::<Vec<_>>();
    assert!(
        violations.is_empty(),
        "public error enum(s) carry #[non_exhaustive]. RST-0006 closes them here; \
         drop the attribute from each occurrence:\n{}",
        violations.join("\n")
    );
}

#[test]
fn visibility_restricted_enum_is_not_flagged() {
    let flagged = flagged_in(
        "#[non_exhaustive] pub(crate) enum HiddenError { A }\n\
         impl std::error::Error for HiddenError {}",
    );
    assert!(
        flagged.is_empty(),
        "a `pub(crate)` enum with #[non_exhaustive] and a manual Error impl is not part of the \
         public error surface and must not be flagged: {flagged:?}"
    );
}

#[test]
fn derive_shape_is_flagged() {
    let flagged = flagged_in(
        "#[non_exhaustive] #[derive(Debug, thiserror::Error)] pub enum DerivedError { A }",
    );
    assert_eq!(flagged, [("DerivedError".to_owned(), ErrorShape::Derived)]);
}

#[test]
fn manual_shape_is_flagged() {
    let flagged = flagged_in(
        "#[non_exhaustive] pub enum ManualError { A }\n\
         impl std::error::Error for ManualError {}",
    );
    assert_eq!(flagged, [("ManualError".to_owned(), ErrorShape::Manual)]);
}

#[test]
fn core_manual_shape_is_flagged() {
    let flagged = flagged_in(
        "#[non_exhaustive] pub enum CoreManualError { A }\n\
         impl core::error::Error for CoreManualError {}",
    );
    assert_eq!(
        flagged,
        [("CoreManualError".to_owned(), ErrorShape::Manual)],
        "a manual `core::error::Error` impl with no derive must reach the manual branch; this \
         fixture cannot pass through the derive branch"
    );
}

#[test]
fn derive_and_manual_shape_is_flagged_once() {
    let flagged = flagged_in(
        "#[non_exhaustive] #[derive(Debug, thiserror::Error)] pub enum BothError { A }\n\
         impl core::error::Error for BothError {}",
    );
    assert_eq!(flagged, [("BothError".to_owned(), ErrorShape::Derived)]);
}

#[test]
fn cfg_test_module_is_not_flagged() {
    let flagged = flagged_in(
        "#[cfg(test)] mod tests {\n\
             #[non_exhaustive] pub enum TestOnlyError { A }\n\
             impl std::error::Error for TestOnlyError {}\n\
         }",
    );
    assert!(
        flagged.is_empty(),
        "a test-only module cannot introduce a public error enum, so nothing inside it is \
         flagged: {flagged:?}"
    );
}

#[test]
fn non_error_non_exhaustive_enum_is_not_flagged() {
    let flagged = flagged_in("#[non_exhaustive] pub enum Marker { A }");
    assert!(
        flagged.is_empty(),
        "#[non_exhaustive] is only a policy violation on an error type: {flagged:?}"
    );
}

#[test]
fn manual_impl_for_a_different_enum_does_not_flag_the_wrong_one() {
    let flagged = flagged_in(
        "#[non_exhaustive] pub enum Alpha { A }\n\
         #[non_exhaustive] pub enum Beta { B }\n\
         impl std::error::Error for Beta {}",
    );
    assert_eq!(
        flagged,
        [("Beta".to_owned(), ErrorShape::Manual)],
        "manual Error impls must be matched by the self type's last segment, not by file"
    );
}

#[test]
fn short_error_trait_path_is_flagged() {
    let flagged = flagged_in(
        "use std::error::Error;\n\
         #[non_exhaustive] pub enum FooError { A }\n\
         impl Error for FooError {}",
    );
    assert_eq!(
        flagged,
        [("FooError".to_owned(), ErrorShape::Manual)],
        "an `Error` impl reached through a short imported path must still count as an error type"
    );
}

#[test]
fn suffixed_error_trait_path_is_flagged() {
    let flagged = flagged_in(
        "use std::error::Error as StdError;\n\
         #[non_exhaustive] pub enum AliasedError { A }\n\
         impl StdError for AliasedError {}",
    );
    assert_eq!(
        flagged,
        [("AliasedError".to_owned(), ErrorShape::Manual)],
        "a trait whose last path segment ends in `Error` is treated as an error trait"
    );
}

#[test]
fn cross_file_manual_impl_is_flagged() {
    let flagged = flagged_across(&[
        "#[non_exhaustive] pub enum FarError { A }",
        "impl std::error::Error for FarError {}",
    ]);
    assert_eq!(
        flagged,
        [("FarError".to_owned(), ErrorShape::Manual)],
        "manual impls are collected across all sources before classification, so an impl in one \
         file must flag an enum declared in another"
    );
}

#[test]
fn rule_param_live_control_holds() {
    let root = manifest_dir();
    let source = parse_source(&root, &root.join("src/config.rs"));
    let mut enums = Vec::new();
    collect_enums(&source.ast.items, &mut enums);
    let rule_param = enums
        .iter()
        .find(|item_enum| item_enum.ident == "RuleParam")
        .expect(
            "src/config.rs must declare `enum RuleParam`; the live control that keeps the public \
             baseline honest has lost its subject and needs re-deriving",
        );
    assert!(
        matches!(rule_param.vis, syn::Visibility::Restricted(_)),
        "`RuleParam` is no longer visibility-restricted; if it became `pub` the closed-error \
         surface changed and this control must be re-derived: {:?}",
        rule_param.vis
    );
    assert!(
        rule_param
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("non_exhaustive")),
        "`RuleParam` no longer carries #[non_exhaustive]; the control needs re-deriving to pin a \
         real enum"
    );
    let scan = scan_live();
    assert!(
        !derives_error(rule_param) && !scan.manual_impls.contains("RuleParam"),
        "`RuleParam` now implements an error trait, in `src/config.rs` or elsewhere in `src/`; \
         the control needs re-deriving because its non-error premise no longer holds"
    );
}

fn flagged_across(sources: &[&str]) -> Vec<(String, ErrorShape)> {
    let asts: Vec<syn::File> = sources
        .iter()
        .map(|source| syn::parse_file(source).expect("classifier fixture must parse"))
        .collect();
    let mut manual_impls = BTreeSet::new();
    for ast in &asts {
        collect_manual_error_types(&ast.items, &mut manual_impls);
    }
    let mut flagged = Vec::new();
    for ast in &asts {
        let mut enums = Vec::new();
        collect_enums(&ast.items, &mut enums);
        for item_enum in enums {
            if let Some(shape) = classify(item_enum, &manual_impls) {
                flagged.push((item_enum.ident.to_string(), shape));
            }
        }
    }
    flagged
}

fn flagged_in(source: &str) -> Vec<(String, ErrorShape)> {
    let ast = syn::parse_file(source).expect("classifier fixture must parse");
    let mut manual_impls = BTreeSet::new();
    collect_manual_error_types(&ast.items, &mut manual_impls);
    let mut enums = Vec::new();
    collect_enums(&ast.items, &mut enums);
    enums
        .into_iter()
        .filter_map(|item_enum| {
            classify(item_enum, &manual_impls).map(|shape| (item_enum.ident.to_string(), shape))
        })
        .collect()
}

fn scan_live() -> Scan {
    let root = manifest_dir();
    let src = root.join("src");
    let mut paths: Vec<PathBuf> = Vec::new();
    rust_sources(&src, &mut paths)
        .unwrap_or_else(|error| panic!("{}: source tree unreadable: {error}", src.display()));
    paths.sort();

    let sources: Vec<SourceFile> = paths.iter().map(|path| parse_source(&root, path)).collect();

    let mut manual_impls = BTreeSet::new();
    for source in &sources {
        collect_manual_error_types(&source.ast.items, &mut manual_impls);
    }

    let mut occurrences = Vec::new();
    let mut all_enums: usize = 0;
    let mut pub_enums: usize = 0;
    for source in &sources {
        let mut enums = Vec::new();
        collect_enums(&source.ast.items, &mut enums);
        all_enums += enums.len();
        pub_enums += enums
            .iter()
            .filter(|item_enum| matches!(item_enum.vis, syn::Visibility::Public(_)))
            .count();
        for item_enum in enums {
            if let Some(shape) = classify(item_enum, &manual_impls) {
                occurrences.push(Occurrence {
                    path: source.relative.clone(),
                    line: declaration_line(source, item_enum),
                    enum_name: item_enum.ident.to_string(),
                    shape,
                });
            }
        }
    }

    Scan {
        files: sources.len(),
        pub_enums,
        all_enums,
        manual_impls,
        occurrences,
    }
}

fn parse_source(root: &Path, path: &Path) -> SourceFile {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{}: source unreadable: {error}", path.display()));
    let ast = syn::parse_file(&text)
        .unwrap_or_else(|error| panic!("{}: source does not parse: {error}", path.display()));
    let relative = path
        .strip_prefix(root)
        .unwrap_or_else(|error| panic!("{}: not under {}: {error}", path.display(), root.display()))
        .to_string_lossy()
        .into_owned();
    SourceFile {
        relative,
        text,
        ast,
    }
}

fn declaration_line(source: &SourceFile, item_enum: &syn::ItemEnum) -> usize {
    let name = item_enum.ident.to_string();
    let needle = format!("pub enum {name}");
    let offset = source
        .text
        .lines()
        .position(|line| {
            line.find(&needle).is_some_and(|index| {
                matches!(
                    line[index + needle.len()..].chars().next(),
                    None | Some(' ' | '{' | '<')
                )
            })
        })
        .unwrap_or_else(|| {
            panic!(
                "{}: cannot locate the declaration line of {name}; the guard must not report a \
                 finding it cannot point at",
                source.relative
            )
        });
    offset + 1
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "syn::Item has many variants; only Enum and Mod can hold an enum declaration"
)]
fn collect_enums<'a>(items: &'a [syn::Item], out: &mut Vec<&'a syn::ItemEnum>) {
    for item in items {
        match item {
            syn::Item::Enum(item_enum) => out.push(item_enum),
            syn::Item::Mod(module) => {
                if module.attrs.iter().any(cfg_false_without_test) {
                    continue;
                }
                if let Some((_, inner)) = &module.content {
                    collect_enums(inner, out);
                }
            }
            _ => {}
        }
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "syn::Item has many variants; only Impl and Mod can carry an Error implementation"
)]
fn collect_manual_error_types(items: &[syn::Item], out: &mut BTreeSet<String>) {
    for item in items {
        match item {
            syn::Item::Impl(item_impl) => {
                if item_impl.attrs.iter().any(cfg_false_without_test) {
                    continue;
                }
                if let Some((path, _)) = &item_impl.trait_
                    && is_error_trait_path(path)
                    && let Some(name) = self_type_name(&item_impl.self_ty)
                {
                    out.insert(name);
                }
            }
            syn::Item::Mod(module) => {
                if module.attrs.iter().any(cfg_false_without_test) {
                    continue;
                }
                if let Some((_, inner)) = &module.content {
                    collect_manual_error_types(inner, out);
                }
            }
            _ => {}
        }
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "syn::Type has many variants; only Path can name an implementation's self type"
)]
fn self_type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => Some(path.path.segments.last()?.ident.to_string()),
        _ => None,
    }
}

fn is_error_trait_path(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident.to_string().ends_with("Error"))
}

fn classify(item_enum: &syn::ItemEnum, manual_impls: &BTreeSet<String>) -> Option<ErrorShape> {
    if !matches!(item_enum.vis, syn::Visibility::Public(_))
        || item_enum.attrs.iter().any(cfg_false_without_test)
        || !item_enum
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("non_exhaustive"))
    {
        return None;
    }
    if derives_error(item_enum) {
        return Some(ErrorShape::Derived);
    }
    manual_impls
        .contains(&item_enum.ident.to_string())
        .then_some(ErrorShape::Manual)
}

fn derives_error(item_enum: &syn::ItemEnum) -> bool {
    item_enum
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .any(|attr| {
            attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths.iter().any(|path| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Error")
                })
            })
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CfgTruth {
    False,
    True,
    Unknown,
}

impl syn::parse::Parse for CfgTruth {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        use syn::ext::IdentExt;

        if input.peek(syn::LitBool) {
            return Ok(if input.parse::<syn::LitBool>()?.value {
                Self::True
            } else {
                Self::False
            });
        }
        let name = input.call(syn::Ident::parse_any)?.unraw().to_string();
        if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            input.parse::<syn::LitStr>()?;
            return Ok(Self::Unknown);
        }
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let values = content.parse_terminated(Self::parse, syn::Token![,])?;
            return match name.as_str() {
                "all" | "any" => {
                    let decisive = if name == "all" {
                        Self::False
                    } else {
                        Self::True
                    };
                    Ok(if values.iter().any(|value| *value == decisive) {
                        decisive
                    } else if values.iter().any(|value| *value == Self::Unknown) {
                        Self::Unknown
                    } else if name == "all" {
                        Self::True
                    } else {
                        Self::False
                    })
                }
                "not" if values.len() == 1 => Ok(match values[0] {
                    Self::False => Self::True,
                    Self::True => Self::False,
                    Self::Unknown => Self::Unknown,
                }),
                _ => Err(input.error("expected all(...), any(...) or not(<one predicate>)")),
            };
        }
        Ok(if name == "test" {
            Self::False
        } else {
            Self::Unknown
        })
    }
}

fn cfg_false_without_test(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("cfg")
        && matches!(
            attr.parse_args_with(|input: syn::parse::ParseStream<'_>| {
                let value = input.parse::<CfgTruth>()?;
                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                }
                Ok(value)
            }),
            Ok(CfgTruth::False)
        )
}
