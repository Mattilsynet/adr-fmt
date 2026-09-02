//! ADR template and link-integrity validator — library surface.
//!
//! Ships as both a binary (`adr-fmt`) and a library (`adr_fmt`); the
//! binary is a thin wrapper over [`run`] so downstream consumers
//! (e.g. `adr-srv`) can reuse parsing, linting, and navigation
//! without spawning a subprocess.
//!
//! # Modes
//!
//! ```text
//! adr-fmt                     # default: print governance guidelines
//! adr-fmt --lint              # lint all ADRs
//! adr-fmt --refs <ADR_ID>     # ADRs that cite the target
//! adr-fmt --context <CRATE>   # decision rules for a crate
//! adr-fmt --tree [DOMAIN]     # domain tree overview
//! ```
//!
//! Corpus discovery walks up from CWD for an `adr-fmt.toml` with a
//! valid `[corpus]` table; no CLI override (SSOT per AFM-0001).
//!
//! Exit codes: `0` — analysis complete (warnings only, or clean);
//! `1` — infrastructure error or lint errors detected.
//!
//! CLI surface frozen for v0.1 per AFM-0001. Library API follows
//! AFM-0026 / CHE-0030: modules private, minimum re-export set for
//! `adr-srv` via a flat `pub use` block (oracle summary bd
//! `adr-fmt-d7ao`).

#![forbid(unsafe_code)]

mod config;
mod containment;
mod context;
mod guidelines;
mod index;
mod model;
mod nav;
mod output;
mod parser;
mod refs;
mod report;
mod rules;

pub use config::{Config, LoadError, ResolveCorpusError, load_quiet, resolve_corpus_root};
pub use containment::{ContainmentError, contained_join, contained_join_optional};
pub use model::{
    AdrId, AdrIdError, AdrRecord, DomainDir, RelVerb, Relationship, Status, Tier, parse_adr_id,
};
pub use parser::{ParseError, ParseOutcome, parse_domain, parse_stale};
pub use report::{Diagnostic, Severity};

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use clap::Parser;

/// ADR template and link-integrity validator.
#[derive(Parser)]
#[command(name = "adr-fmt", version)]
struct Cli {
    /// Lint all ADRs, report diagnostics to stdout
    #[arg(long, group = "mode")]
    lint: bool,

    /// List ADRs that cite the target via References or Supersedes
    #[arg(long, value_name = "ADR_ID", group = "mode")]
    refs: Option<String>,

    /// Show decision rules applicable to a crate
    #[arg(long, value_name = "CRATE", group = "mode")]
    context: Option<String>,

    /// Print domain tree (optionally filtered by domain prefix)
    #[arg(long, value_name = "DOMAIN", num_args = 0..=1, default_missing_value = "", group = "mode")]
    tree: Option<String>,
}

enum Mode {
    Guidelines,
    Lint,
    Refs(String),
    Context(String),
    Tree(Option<String>),
}

impl Cli {
    fn mode(self) -> Mode {
        if let Some(adr_id) = self.refs {
            Mode::Refs(adr_id)
        } else if let Some(crate_name) = self.context {
            Mode::Context(crate_name)
        } else if let Some(domain_filter) = self.tree {
            Mode::Tree(if domain_filter.is_empty() {
                None
            } else {
                Some(domain_filter)
            })
        } else if self.lint {
            Mode::Lint
        } else {
            Mode::Guidelines
        }
    }
}

/// Library entry-point: parse `args` as the CLI, dispatch, return the exit code.
///
/// The binary [`main`] is a thin wrapper around this function. Future
/// library consumers (e.g. `adr-srv`) call lower-level modules directly
/// (`parser`, `rules`, `nav`); `run` exists primarily to keep the binary
/// surface a one-liner and to provide a top-level smoke-testable entry.
///
/// Dispatch failures are reported by writing to stderr and returning a
/// non-zero exit code, preserving AFM-0001 CLI behaviour bit-for-bit.
/// This function never terminates the calling process: per AFM-0026:R4
/// `src/main.rs` is the only authorised exit site.
///
/// # Errors
///
/// Returns [`clap::Error`] when `args` does not parse as the CLI, and for
/// clap's own `--help` and `--version` display paths. Those two are
/// successes, not failures: their [`clap::Error::exit_code`] is `0`, and
/// a caller that maps the exit code faithfully preserves AFM-0003:R1.
pub fn run<I, T>(args: I) -> Result<i32, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::try_parse_from(args)?;
    let mode = cli.mode();

    let discovery = match discover_marker() {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("error: {msg}");
            return Ok(1);
        }
    };

    if matches!(mode, Mode::Guidelines) {
        return Ok(run_default_mode(discovery));
    }

    let (marker_dir, config) = match discovery {
        ConfigDiscovery::Ready { marker_dir, config } => (marker_dir, config),
        ConfigDiscovery::Invalid {
            marker_path,
            reason,
        } => {
            eprintln!("error: {} is not usable: {reason}", marker_path.display());
            eprintln!(
                "       refusing to fall back to a parent corpus — that would lint a \
                 different corpus and report success"
            );
            return Ok(1);
        }
        ConfigDiscovery::Absent => {
            eprintln!(
                "error: no adr-fmt.toml with a valid [corpus] table found in any parent directory"
            );
            eprintln!("       run from the workspace root, or create adr-fmt.toml there");
            return Ok(1);
        }
    };

    config::emit_legacy_rule_warnings(&config);

    let adr_root = match config::resolve_corpus_root(&marker_dir, &config.corpus) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(1);
        }
    };

    let domain_dirs = match discover_domains(&adr_root, &config) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(1);
        }
    };

    if domain_dirs.is_empty() {
        eprintln!(
            "error: no domain directories found in {}",
            adr_root.display()
        );
        return Ok(1);
    }

    let mut scan = match scan_corpus(&adr_root, &config, &domain_dirs) {
        Ok(scan) => scan,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(1);
        }
    };
    let parse_diagnostics = scan.take_diagnostics();
    let all_records = scan.records();

    let index = match index::CorpusIndex::build(&scan) {
        Ok(idx) => idx,
        Err(dup) => {
            return Ok(report_duplicate_id(
                matches!(mode, Mode::Lint),
                parse_diagnostics,
                all_records.len(),
                &dup,
            ));
        }
    };

    Ok(dispatch_mode(
        &mode,
        all_records,
        &config,
        &domain_dirs,
        &index,
        parse_diagnostics,
    ))
}

fn dispatch_mode(
    mode: &Mode,
    all_records: &[model::AdrRecord],
    config: &Config,
    domain_dirs: &[DomainDir],
    index: &index::CorpusIndex<'_>,
    parse_diagnostics: Vec<report::Diagnostic>,
) -> i32 {
    match mode {
        Mode::Guidelines => 0,
        Mode::Refs(adr_id_str) => {
            let Some(target_id) = parse_adr_id(adr_id_str) else {
                eprintln!(
                    "error: {} is not a valid ADR ID (expected PREFIX-NNNN)",
                    adr_id_str.escape_debug()
                );
                return 1;
            };
            let report = match refs::find_refs(&target_id, index) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("error: {e}");
                    return 1;
                }
            };
            print!("{}", output::render_refs(&report));
            0
        }
        Mode::Context(crate_name) => {
            let groups = match context::context_grouped(crate_name, all_records, config, index) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("error: {e}");
                    return 1;
                }
            };
            print!("{}", output::render_root_groups(crate_name, &groups));
            0
        }
        Mode::Tree(domain_filter) => {
            print!(
                "{}",
                output::render_tree(
                    all_records,
                    domain_dirs,
                    config,
                    domain_filter.as_deref(),
                    index
                )
            );
            0
        }
        Mode::Lint => {
            let mut diagnostics = parse_diagnostics;
            diagnostics.extend(rules::run_all(all_records, config, index));
            print!(
                "{}",
                output::render_diagnostics(&diagnostics, all_records.len())
            );
            0
        }
    }
}

fn report_duplicate_id(
    lint_mode: bool,
    parse_diagnostics: Vec<report::Diagnostic>,
    record_count: usize,
    dup: &index::DuplicateId,
) -> i32 {
    if lint_mode {
        let mut diagnostics = parse_diagnostics;
        diagnostics.push(duplicate_id_diagnostic(dup));
        print!("{}", output::render_diagnostics(&diagnostics, record_count));
        return 0;
    }
    eprintln!(
        "error: duplicate ADR id {} — {} and {} both claim it (AFM-0008:R3 requires a permanent, globally unambiguous id)",
        dup.id,
        dup.paths[0].display(),
        dup.paths[1].display(),
    );
    1
}

fn duplicate_id_diagnostic(dup: &index::DuplicateId) -> report::Diagnostic {
    report::Diagnostic::warning(
        "P004",
        &dup.paths[0],
        0,
        format!(
            "duplicate ADR id {}: also claimed by {} — every AdrId must be \
             globally unambiguous (AFM-0008:R3)",
            dup.id,
            dup.paths[1].display(),
        ),
    )
}

fn run_default_mode(discovery: ConfigDiscovery) -> i32 {
    match discovery {
        ConfigDiscovery::Ready { marker_dir, config } => {
            match config::resolve_corpus_root(&marker_dir, &config.corpus) {
                Ok(_) => {
                    guidelines::print_governance(&config);
                    0
                }
                Err(e) => {
                    eprintln!("error: adr-fmt.toml in {}: {e}", marker_dir.display());
                    eprintln!(
                        "       the config was found but is not usable; fix it rather than re-running setup"
                    );
                    1
                }
            }
        }
        ConfigDiscovery::Invalid {
            marker_path,
            reason,
        } => {
            eprintln!("error: {} is not usable: {reason}", marker_path.display());
            eprintln!(
                "       adr-fmt found this config but cannot use it, so it will not fall back \
                 to a parent corpus"
            );
            1
        }
        ConfigDiscovery::Absent => {
            guidelines::print_setup_guide();
            0
        }
    }
}

fn scan_corpus(
    adr_root: &Path,
    config: &Config,
    domain_dirs: &[DomainDir],
) -> Result<index::ScannedCorpus, String> {
    let mut scan = index::ScannedCorpus::default();

    for dir in domain_dirs {
        scan.absorb(parser::parse_domain(dir).map_err(|e| e.to_string())?);
    }

    let stale_dir = containment::contained_join_optional(adr_root, &config.stale.directory)
        .map_err(|e| format!("stale directory in adr-fmt.toml: {e}"))?;
    if let Some(stale_dir) = stale_dir
        && stale_dir.is_dir()
    {
        scan.absorb(parser::parse_stale(&stale_dir, config).map_err(|e| e.to_string())?);
    }

    Ok(scan)
}

enum ConfigDiscovery {
    Absent,
    Ready {
        marker_dir: PathBuf,
        config: Box<Config>,
    },
    Invalid {
        marker_path: PathBuf,
        reason: String,
    },
}

enum MarkerVerdict {
    Unfit(String),
    Ready {
        marker_dir: PathBuf,
        config: Box<Config>,
    },
    Invalid(String),
}

fn discover_marker() -> Result<ConfigDiscovery, String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("cannot determine current directory: {e}"))?;
    let canon_cwd = std::fs::canonicalize(&cwd).unwrap_or(cwd);
    let mut dir = canon_cwd.as_path();
    loop {
        let candidate = dir.join("adr-fmt.toml");
        if candidate.is_file() {
            match try_marker(dir)? {
                MarkerVerdict::Ready { marker_dir, config } => {
                    return Ok(ConfigDiscovery::Ready { marker_dir, config });
                }
                MarkerVerdict::Invalid(reason) => {
                    return Ok(ConfigDiscovery::Invalid {
                        marker_path: candidate,
                        reason,
                    });
                }
                MarkerVerdict::Unfit(note) => {
                    eprintln!("note: skipping {}: {note}", candidate.display());
                }
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Ok(ConfigDiscovery::Absent),
        }
    }
}

fn try_marker(marker_dir: &Path) -> Result<MarkerVerdict, String> {
    let config = match config::load_quiet(marker_dir) {
        Ok(c) => c,
        Err(config::LoadError::Io(m)) => return Err(m),
        Err(config::LoadError::Parse(m)) => return Ok(MarkerVerdict::Invalid(m)),
        Err(config::LoadError::NotAMarker(m)) => return Ok(MarkerVerdict::Unfit(m)),
    };
    let Ok(corpus_root) = config::resolve_corpus_root(marker_dir, &config.corpus) else {
        return Ok(MarkerVerdict::Unfit(
            "[corpus] root does not resolve within this directory".to_owned(),
        ));
    };
    if !corpus_root.is_dir() {
        return Ok(MarkerVerdict::Unfit(format!(
            "corpus root {} is not a directory",
            corpus_root.display()
        )));
    }

    let mut any_domain_intended = false;
    for d in &config.domains {
        match containment::contained_join_optional(&corpus_root, &d.directory) {
            Err(containment::ContainmentError::MetadataFailed { segment, reason }) => {
                return Ok(MarkerVerdict::Invalid(format!(
                    "domain '{}' directory {}: {reason} — whether this marker \
                     describes the corpus here cannot be determined",
                    d.prefix,
                    segment.escape_debug()
                )));
            }
            Err(_) => any_domain_intended = true,
            Ok(Some(p)) => any_domain_intended |= p.is_dir(),
            Ok(None) => {}
        }
    }
    if !any_domain_intended {
        return Ok(MarkerVerdict::Unfit(
            "no configured domain resolves to an existing directory".to_owned(),
        ));
    }

    let marker_dir = std::fs::canonicalize(marker_dir)
        .map_err(|e| format!("cannot canonicalize {}: {e}", marker_dir.display()))?;
    Ok(MarkerVerdict::Ready {
        marker_dir,
        config: Box::new(config),
    })
}

fn discover_domains(root: &Path, config: &Config) -> Result<Vec<DomainDir>, String> {
    let mut dirs = Vec::new();
    for domain in &config.domains {
        let resolved = containment::contained_join_optional(root, &domain.directory)
            .map_err(|e| format!("domain '{}' directory: {e}", domain.prefix))?;
        if let Some(path) = resolved
            && path.is_dir()
        {
            dirs.push(DomainDir {
                path,
                prefix: domain.prefix.clone(),
                name: domain.name.clone(),
            });
        }
    }
    Ok(dirs)
}

#[cfg(test)]
mod mode_tests {
    use super::{Cli, Mode};
    use clap::Parser;

    fn mode_of(args: &[&str]) -> Mode {
        Cli::parse_from(args).mode()
    }

    #[test]
    fn no_flag_maps_to_guidelines() {
        assert!(matches!(mode_of(&["adr-fmt"]), Mode::Guidelines));
    }

    #[test]
    fn lint_flag_maps_to_lint() {
        assert!(matches!(mode_of(&["adr-fmt", "--lint"]), Mode::Lint));
    }

    #[test]
    fn refs_flag_maps_to_refs_with_payload() {
        assert!(
            matches!(mode_of(&["adr-fmt", "--refs", "AFM-0013"]), Mode::Refs(id) if id == "AFM-0013")
        );
    }

    #[test]
    fn context_flag_maps_to_context_with_payload() {
        assert!(
            matches!(mode_of(&["adr-fmt", "--context", "adr-fmt"]), Mode::Context(c) if c == "adr-fmt")
        );
    }

    #[test]
    fn bare_tree_flag_maps_to_unfiltered_tree() {
        assert!(matches!(mode_of(&["adr-fmt", "--tree"]), Mode::Tree(None)));
    }

    #[test]
    fn tree_flag_with_domain_maps_to_filtered_tree() {
        assert!(
            matches!(mode_of(&["adr-fmt", "--tree", "AFM"]), Mode::Tree(Some(d)) if d == "AFM")
        );
    }
}

#[cfg(test)]
mod discover_marker_tests {
    use super::discover_marker;
    use std::path::PathBuf;
    use std::sync::Mutex;

    static CWD_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct CwdRestoreGuard {
        original: PathBuf,
    }

    impl CwdRestoreGuard {
        fn capture() -> Self {
            Self {
                original: std::env::current_dir().expect("capture current dir"),
            }
        }
    }

    impl Drop for CwdRestoreGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    #[test]
    fn cwd_failure_is_distinct_from_no_marker_found() {
        let _lock = CWD_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _guard = CwdRestoreGuard::capture();

        let vanished = tempfile::tempdir().expect("create temp dir");
        std::env::set_current_dir(vanished.path()).expect("cd into temp dir");
        drop(vanished);

        let result = discover_marker();

        assert!(
            result.is_err(),
            "cwd unavailable must surface as Err, not Ok(None) (no-marker-found)"
        );
    }
}
