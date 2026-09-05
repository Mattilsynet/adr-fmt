//! Configuration loading from `adr-fmt.toml`.
//!
//! The config file lives at the workspace root and is the SSOT discovery
//! marker. It defines the corpus root, domain mappings, stale directory,
//! and optional rule parameter overrides. Rules themselves are hardcoded
//! in the binary. Rationale and judgment guidance live in dedicated ADRs
//! under `docs/adr/adr-fmt/` (see AFM-0001, AFM-0020).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Top-level configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub corpus: CorpusConfig,
    pub stale: StaleConfig,
    pub domains: Vec<DomainConfig>,
    /// Optional rule overrides. If present with full declarations (legacy
    /// format), a deprecation warning is emitted to stderr.
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
}

/// Corpus-root configuration. The `root` value is a relative path from
/// the directory containing `adr-fmt.toml` to the ADR corpus directory.
#[derive(Debug, Deserialize)]
pub struct CorpusConfig {
    pub root: String,
}

/// Stale archive configuration.
#[derive(Debug, Deserialize)]
pub struct StaleConfig {
    pub directory: String,
}

/// Domain definition.
#[derive(Debug, Deserialize)]
pub struct DomainConfig {
    pub prefix: String,
    pub name: String,
    pub directory: String,
    pub description: String,
    pub crates: Vec<String>,
    /// Foundation domains are included with every domain query.
    #[serde(default)]
    pub foundation: bool,
}

/// Rule override entry. Only `id` is required; other fields are optional
/// and used only for parameter overrides or disabling rules.
#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    /// Optional rule parameters (e.g., `min_words = 7`).
    #[serde(default)]
    pub params: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum RuleParam {
    Absent,
    Value(u64),
    Invalid {
        rule_id: String,
        key: String,
        reason: String,
    },
}

impl Config {
    #[must_use]
    pub(crate) fn rule_param_u64(&self, rule_id: &str, key: &str) -> RuleParam {
        let Some(raw) = self
            .rules
            .iter()
            .find(|r| r.id == rule_id)
            .and_then(|r| r.params.get(key))
        else {
            return RuleParam::Absent;
        };
        match raw.as_integer() {
            None => RuleParam::Invalid {
                rule_id: rule_id.to_owned(),
                key: key.to_owned(),
                reason: format!("expected an integer, found {}", raw.type_str()),
            },
            Some(v) => match u64::try_from(v) {
                Ok(v) => RuleParam::Value(v),
                Err(_) => RuleParam::Invalid {
                    rule_id: rule_id.to_owned(),
                    key: key.to_owned(),
                    reason: format!("{v} is out of range for a non-negative budget"),
                },
            },
        }
    }
}

/// Raw deserialisation target for `adr-fmt.toml`. Private by design:
/// it is the only shape TOML is decoded into, so every [`Config`]
/// produced by [`load_quiet`] has passed validation. Field types match
/// [`Config`] exactly; the nested types are shared, not duplicated.
#[derive(Debug, Deserialize)]
struct RawConfig {
    corpus: CorpusConfig,
    stale: StaleConfig,
    domains: Vec<DomainConfig>,
    #[serde(default)]
    rules: Vec<RuleConfig>,
}

fn reject_duplicate_rule_ids(rules: &[RuleConfig]) -> Result<(), LoadError> {
    let mut seen = BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.id.as_str()) {
            return Err(LoadError::DuplicateRuleId(rule.id.clone()));
        }
    }
    Ok(())
}

impl TryFrom<RawConfig> for Config {
    type Error = LoadError;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        reject_duplicate_rule_ids(&raw.rules)?;
        Ok(Self {
            corpus: raw.corpus,
            stale: raw.stale,
            domains: raw.domains,
            rules: raw.rules,
        })
    }
}

/// Load configuration from `adr-fmt.toml` in the marker directory,
/// suppressing the legacy-rule deprecation warning.
///
/// `marker_dir` is the directory containing `adr-fmt.toml` (typically
/// the workspace root). Used by walk-up discovery so warnings from
/// skipped (non-selected) markers do not pollute stderr.
///
/// # Errors
///
/// Returns [`LoadError::Io`] when `adr-fmt.toml` cannot be read.
/// Returns [`LoadError::Parse`] when TOML parsing fails.
/// Returns [`LoadError::NotAMarker`] when the file parses as TOML but
/// declares no `[corpus]` table.
/// Returns [`LoadError::DuplicateRuleId`] when the file declares the
/// same `[[rules]] id` more than once. This applies to the quiet path
/// too: a marker whose configuration is contradictory is broken, and
/// discovery must stop at it rather than walk past it — the same
/// treatment [`LoadError::Parse`] already gets.
pub fn load_quiet(marker_dir: &Path) -> Result<Config, LoadError> {
    load_inner_typed(marker_dir)
}

/// Distinguishes how a marker load failed. `Io` indicates the file
/// existed but could not be read (permission denied, etc.) — discovery
/// must treat this as a hard error rather than skip. `Parse` covers
/// malformed TOML — the file claims to be a marker but is broken, so
/// discovery must stop rather than walk past it. `NotAMarker` covers a
/// well-formed TOML file with no `[corpus]` table — it makes no marker
/// claim, so discovery may skip it and continue walking up.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadError {
    Io(String),
    Parse(String),
    /// The file parsed as TOML but declares no `[corpus]` table, so it
    /// does not claim to be an adr-fmt marker at all. Distinct from
    /// [`LoadError::Parse`]: discovery may walk past a non-marker, but
    /// a broken marker must not be silently skipped.
    NotAMarker(String),
    /// The file parsed as TOML and claims to be a marker, but declares
    /// the same `[[rules]] id` more than once. Carries the offending
    /// rule id. Distinct from [`LoadError::Parse`]: the TOML is
    /// well-formed, the configuration it expresses is not.
    DuplicateRuleId(String),
}

impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::Io(msg) => write!(f, "adr-fmt config I/O error: {msg}"),
            LoadError::Parse(msg) => write!(f, "adr-fmt config parse error: {msg}"),
            LoadError::NotAMarker(msg) => write!(f, "not an adr-fmt marker: {msg}"),
            LoadError::DuplicateRuleId(rule_id) => write!(
                f,
                "adr-fmt config validation error: adr-fmt.toml declares \
                 [[rules]] id = \"{rule_id}\" more than once; each rule id \
                 may be declared at most once, otherwise parameter lookups \
                 silently use the first declaration"
            ),
        }
    }
}

impl std::error::Error for LoadError {}

fn load_inner_typed(marker_dir: &Path) -> Result<Config, LoadError> {
    let config_path = marker_dir.join("adr-fmt.toml");

    let content = std::fs::read_to_string(&config_path).map_err(|e| {
        LoadError::Io(format!(
            "cannot read {}: {e}\n       adr-fmt.toml is required at the workspace root",
            config_path.display()
        ))
    })?;

    let config: RawConfig = toml::from_str(&content).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("missing field `corpus`") {
            LoadError::NotAMarker(format!(
                "{}: missing required `[corpus]` table\n\
                 \n\
                 Example:\n\
                 \n\
                     [corpus]\n\
                     root = \"docs/adr\"\n",
                config_path.display()
            ))
        } else {
            LoadError::Parse(format!("failed to parse {}: {e}", config_path.display()))
        }
    })?;

    Config::try_from(config)
}

/// Resolve the corpus root path relative to the marker directory.
///
/// Applies strict containment via [`crate::containment::contained_join`]:
/// the configured `corpus.root` must be a relative path with no parent-
/// traversal components, and the canonical target must be a descendant
/// of the canonical marker directory. The corpus directory must exist.
///
/// # Errors
///
/// Returns [`ResolveCorpusError::Containment`] when `corpus.root` fails
/// containment validation, canonicalization, or descendant checks.
pub fn resolve_corpus_root(
    marker_dir: &Path,
    corpus: &CorpusConfig,
) -> Result<PathBuf, ResolveCorpusError> {
    crate::containment::contained_join(marker_dir, &corpus.root).map_err(ResolveCorpusError::from)
}

/// Failure resolving `[corpus] root` from the marker directory.
///
/// `resolve_corpus_root` fails only via
/// [`crate::containment::contained_join`], so this taxonomy wraps
/// [`crate::containment::ContainmentError`] rather than re-deriving its
/// variants.
#[derive(Debug)]
#[non_exhaustive]
pub enum ResolveCorpusError {
    Containment(crate::containment::ContainmentError),
}

impl core::fmt::Display for ResolveCorpusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Containment(e) => write!(f, "[corpus] root: {e}"),
        }
    }
}

impl std::error::Error for ResolveCorpusError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Containment(e) => Some(e),
        }
    }
}

impl From<crate::containment::ContainmentError> for ResolveCorpusError {
    fn from(e: crate::containment::ContainmentError) -> Self {
        Self::Containment(e)
    }
}

/// Emit deprecation warnings if config contains legacy full rule declarations.
///
/// Legacy format: rules with `category` and `description` fields populated.
/// New format: only `id` and optional `params` for overrides.
///
/// Public so `main.rs` can fire it once on the *selected* marker after
/// walk-up discovery — the walk-up itself uses [`load_quiet`], which
/// suppresses the warning for skipped (non-selected) markers so stderr
/// stays focused on the marker the user actually committed to.
pub fn emit_legacy_rule_warnings(config: &Config) {
    let legacy_count = config
        .rules
        .iter()
        .filter(|r| !r.category.is_empty() && !r.description.is_empty())
        .count();

    if legacy_count > 0 {
        eprintln!("warning: adr-fmt.toml contains {legacy_count} legacy rule declaration(s)");
        eprintln!("         Rules are now hardcoded in the binary. Only parameter overrides");
        eprintln!("         are needed in config. Remove `category` and `description` fields.");
        eprintln!("         Example override: [[rules]]");
        eprintln!("         id = \"T015\"");
        eprintln!("         params = {{ min_words = 7, max_words = 100 }}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config_no_rules() {
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
crates = ["example-core"]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.stale.directory, "stale");
        assert_eq!(config.domains.len(), 1);
        assert_eq!(config.domains[0].prefix, "CHE");
        assert_eq!(config.domains[0].crates, vec!["example-core"]);
        assert!(config.rules.is_empty());
    }

    #[test]
    fn parse_config_with_overrides() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 7, max_words = 50 }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].id, "T015");
        assert_eq!(
            config.rule_param_u64("T015", "min_words"),
            RuleParam::Value(7)
        );
        assert_eq!(
            config.rule_param_u64("T015", "max_words"),
            RuleParam::Value(50)
        );
    }

    #[test]
    fn parse_multi_domain_config() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "COM"
name = "Common"
directory = "common"
description = "Cross-cutting"
crates = []

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Architecture"
crates = ["example-core", "example-gateway"]
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.domains.len(), 2);
        assert_eq!(config.domains[0].prefix, "COM");
        assert!(config.domains[0].crates.is_empty());
        assert_eq!(config.domains[1].crates.len(), 2);
    }

    #[test]
    fn parse_rule_with_params() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 10 }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rules[0].id, "T015");
        let min_words = config.rule_param_u64("T015", "min_words");
        assert_eq!(min_words, RuleParam::Value(10));
    }

    #[test]
    fn rule_param_missing_returns_none() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.rule_param_u64("T020", "min_words"),
            RuleParam::Absent
        );
        assert_eq!(config.rule_param_u64("MISSING", "key"), RuleParam::Absent);
    }

    #[test]
    fn rule_param_u64_distinguishes_absent_from_malformed() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = { min_words = 7, max_words = "seven", negative = -3 }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert!(matches!(
            config.rule_param_u64("T015", "min_words"),
            RuleParam::Value(7)
        ));
        assert!(
            matches!(config.rule_param_u64("T015", "absent"), RuleParam::Absent),
            "an absent key is not a malformed one"
        );
        assert!(
            matches!(config.rule_param_u64("MISSING", "key"), RuleParam::Absent),
            "an absent rule is not a malformed one"
        );
        assert!(
            matches!(
                config.rule_param_u64("T015", "max_words"),
                RuleParam::Invalid { .. }
            ),
            "a wrong-typed value must not read as absent"
        );
        assert!(
            matches!(
                config.rule_param_u64("T015", "negative"),
                RuleParam::Invalid { .. }
            ),
            "an out-of-range value must not read as absent"
        );
    }

    #[test]
    fn missing_corpus_table_is_not_a_marker_but_bad_toml_is_malformed() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("adr-fmt.toml"),
            "[stale]\ndirectory = \"x\"\n",
        )
        .expect("write");
        assert!(
            matches!(load_quiet(dir.path()), Err(LoadError::NotAMarker(_))),
            "a toml without [corpus] does not claim to be an adr-fmt marker"
        );

        let dir2 = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir2.path().join("adr-fmt.toml"), "not valid toml ===").expect("write");
        assert!(
            matches!(load_quiet(dir2.path()), Err(LoadError::Parse(_))),
            "unparseable toml is a broken marker, not a non-marker"
        );
    }

    fn duplicate_config(second_id: &str) -> String {
        format!(
            r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T015"
params = {{ min_words = 7 }}

[[rules]]
id = "{second_id}"
params = {{ min_words = 99 }}
"#
        )
    }

    #[test]
    fn duplicate_rule_id_is_a_load_error_not_a_diagnostic() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("adr-fmt.toml"), duplicate_config("T015")).expect("write");

        let err = load_quiet(dir.path()).expect_err("a duplicate rule id must fail the load");
        assert!(
            matches!(&err, LoadError::DuplicateRuleId(id) if id == "T015"),
            "got: {err:?}"
        );

        let rendered = err.to_string();
        assert!(
            rendered.contains("T015") && rendered.contains("adr-fmt.toml"),
            "Display must name the rule id and the relative segment; got: {rendered}"
        );
        assert!(
            !rendered.contains(dir.path().to_str().expect("utf-8 tempdir")),
            "Display must not leak the absolute marker path (AFM-0028:R2); got: {rendered}"
        );
    }

    #[test]
    fn distinct_rule_ids_still_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("adr-fmt.toml"), duplicate_config("T020")).expect("write");

        let config = load_quiet(dir.path()).expect("distinct rule ids are valid");
        assert_eq!(config.rules.len(), 2);
        assert_eq!(
            config.rule_param_u64("T020", "min_words"),
            RuleParam::Value(99)
        );
    }

    #[test]
    fn missing_required_field_fails() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
# missing directory and description
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn foundation_flag_defaults_to_false() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.domains[0].foundation);
    }

    #[test]
    fn foundation_flag_true_deserializes() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "COM"
name = "Common"
directory = "common"
description = "Cross-cutting"
crates = []
foundation = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.domains[0].foundation);
    }

    #[test]
    fn legacy_format_still_parses() {
        let toml_str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []

[[rules]]
id = "T020"
category = "template"
description = "Reference load"

[[rules]]
id = "T002"
category = "template"
description = "Date field present"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rules.len(), 2);
    }

    #[test]
    fn missing_corpus_table_emits_clear_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let toml_str = r#"
[stale]
directory = "stale"

[[domains]]
prefix = "CHE"
name = "Cherry"
directory = "cherry"
description = "Test"
crates = []
"#;
        std::fs::write(dir.path().join("adr-fmt.toml"), toml_str).unwrap();
        let err = match load_quiet(dir.path()).unwrap_err() {
            LoadError::NotAMarker(m) => m,
            LoadError::Parse(m) => panic!("expected NotAMarker, got Parse: {m}"),
            LoadError::Io(m) => panic!("expected NotAMarker, got Io: {m}"),
            LoadError::DuplicateRuleId(id) => {
                panic!("expected NotAMarker, got DuplicateRuleId: {id}")
            }
        };
        assert!(
            err.contains("`[corpus]`"),
            "error must name the [corpus] table; got: {err}"
        );
        assert!(
            err.contains("root = \"docs/adr\""),
            "error must show example; got: {err}"
        );
    }

    #[test]
    fn resolve_corpus_root_returns_canonical_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs/adr")).unwrap();
        let corpus = CorpusConfig {
            root: "docs/adr".to_owned(),
        };
        let resolved = resolve_corpus_root(dir.path(), &corpus).expect("resolves");
        assert!(resolved.ends_with("docs/adr"));
    }

    #[test]
    fn resolve_corpus_root_rejects_absolute() {
        let dir = tempfile::tempdir().expect("tempdir");
        let corpus = CorpusConfig {
            root: "/etc".to_owned(),
        };
        let err = resolve_corpus_root(dir.path(), &corpus).unwrap_err();
        assert!(
            matches!(
                err,
                ResolveCorpusError::Containment(crate::containment::ContainmentError::Absolute(_))
            ),
            "got: {err:?}"
        );
    }

    #[test]
    fn resolve_corpus_root_rejects_parent_traversal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let corpus = CorpusConfig {
            root: "../escape".to_owned(),
        };
        let err = resolve_corpus_root(dir.path(), &corpus).unwrap_err();
        assert!(
            matches!(
                err,
                ResolveCorpusError::Containment(
                    crate::containment::ContainmentError::ParentTraversal(_)
                )
            ),
            "got: {err:?}"
        );
    }
}
