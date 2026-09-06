//! Library smoke/API probes: `adr_fmt::run` callable; root re-exports resolve.
//!
//! `--help`/`--version`/infrastructure-failure termination guards spawn ignored
//! child probes in this executable: in-process assertions cannot survive
//! `process::exit`. Parents require successful exit and a sentinel printed
//! after `run` returns; non-zero exit or missing sentinel fails.
//!
//! `context`, `nav`, `output`, `refs`, `rules`, `guidelines` are private
//! per CHE-0030 (Flat Public API via Private Modules): consumers MUST NOT name
//! them; no probes cover those paths.
//!
//! Binary regression coverage lives in `tests/integration.rs`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

const SENTINEL: &str = "ADR_FMT_RUN_RETURNED";

fn spawn_child_probe(test_name: &str) -> String {
    let exe = std::env::current_exe().expect("test executable path");
    let output = Command::new(exe)
        .args([
            "--exact",
            test_name,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .expect("spawn child probe");
    assert!(
        output.status.success(),
        "child probe {test_name} did not exit successfully: {:?}",
        output.status
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
#[ignore = "spawned by the termination-guard parent test"]
fn child_probe_help() {
    let argv: Vec<OsString> = vec![OsString::from("adr-fmt"), OsString::from("--help")];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    println!("{SENTINEL} help {outcome:?}");
}

#[test]
#[ignore = "spawned by the termination-guard parent test"]
fn child_probe_version() {
    let argv: Vec<OsString> = vec![OsString::from("adr-fmt"), OsString::from("--version")];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    println!("{SENTINEL} version {outcome:?}");
}

#[test]
#[ignore = "spawned by the termination-guard parent test"]
fn child_probe_infrastructure_failure() {
    let argv: Vec<OsString> = vec![
        OsString::from("adr-fmt"),
        OsString::from("--refs"),
        OsString::from("INVALID"),
    ];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    println!("{SENTINEL} refs {outcome:?}");
}

#[test]
fn run_default_mode_via_lib_api_returns_zero() {
    let argv: Vec<OsString> = vec![OsString::from("adr-fmt")];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    assert!(
        outcome.is_ok(),
        "default-mode run should succeed: {outcome:?}"
    );
}

#[test]
fn help_returns_to_caller_instead_of_terminating_the_process() {
    let stdout = spawn_child_probe("child_probe_help");
    assert!(
        stdout.contains(&format!("{SENTINEL} help Ok(())")),
        "`run` must return control to the caller with a success result for --help \
         (AFM-0003:R1); the post-call sentinel was absent, which means the \
         process terminated inside `run`. child stdout:\n{stdout}"
    );
}

#[test]
fn version_returns_to_caller_instead_of_terminating_the_process() {
    let stdout = spawn_child_probe("child_probe_version");
    assert!(
        stdout.contains(&format!("{SENTINEL} version Ok(())")),
        "`run` must return control to the caller with a success result for --version \
         (AFM-0003:R1); the post-call sentinel was absent, which means the \
         process terminated inside `run`. child stdout:\n{stdout}"
    );
}

#[test]
fn infrastructure_failure_returns_to_caller_instead_of_terminating_the_process() {
    let stdout = spawn_child_probe("child_probe_infrastructure_failure");
    assert!(
        stdout.contains(&format!("{SENTINEL} refs Err(Infrastructure)")),
        "`run` must return `Err(RunError::Infrastructure)` to the caller rather \
         than calling `process::exit(1)` (AFM-0026:R10); the post-call sentinel \
         was absent, which means the process terminated inside `run`. child \
         stdout:\n{stdout}"
    );
}

#[test]
fn parse_error_returns_to_caller_instead_of_terminating_the_process() {
    let argv: Vec<OsString> = vec![
        OsString::from("adr-fmt"),
        OsString::from("--no-such-flag-exists"),
    ];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    assert!(
        matches!(outcome, Err(adr_fmt::RunError::Usage)),
        "an unknown flag is a CLI usage failure, not an infrastructure one: {outcome:?}"
    );
}

#[test]
fn mutually_exclusive_modes_return_a_conflict_error() {
    let argv: Vec<OsString> = vec![
        OsString::from("adr-fmt"),
        OsString::from("--lint"),
        OsString::from("--tree"),
    ];
    let outcome: Result<(), adr_fmt::RunError> = adr_fmt::run(argv);
    assert!(
        matches!(outcome, Err(adr_fmt::RunError::Usage)),
        "clap-declared exclusivity must still reject as a usage failure: {outcome:?}"
    );
}

#[test]
fn run_error_implements_the_public_error_trait_obligation() {
    let err = adr_fmt::run(vec![
        OsString::from("adr-fmt"),
        OsString::from("--no-such-flag-exists"),
    ])
    .expect_err("an unknown flag must fail");

    let as_error: &dyn std::error::Error = &err;
    assert!(
        !as_error.to_string().is_empty(),
        "Display must render a non-empty, human-readable message (AFM-0028:R2)"
    );
    assert!(!format!("{err:?}").is_empty(), "Debug must render");
}

#[test]
fn lib_api_modules_resolve() {
    let _: adr_fmt::Severity = adr_fmt::Severity::Warning;
    let _: adr_fmt::Diagnostic =
        adr_fmt::Diagnostic::warning("T999", Path::new("probe.md"), 1, String::from("probe"));

    let _: adr_fmt::Tier = adr_fmt::Tier::A;
    let _: adr_fmt::DomainDir = adr_fmt::DomainDir {
        path: PathBuf::from("/tmp/probe"),
        prefix: String::from("PRB"),
        name: String::from("probe"),
    };
    let _: Option<adr_fmt::AdrId> = adr_fmt::parse_adr_id("PRB-0001");
    let _: adr_fmt::Status = adr_fmt::Status::Accepted;
    let _: adr_fmt::RelVerb = adr_fmt::RelVerb::References;
    let _: fn() -> Vec<adr_fmt::Relationship> = || Vec::new();

    let _: Result<PathBuf, adr_fmt::ContainmentError> =
        adr_fmt::contained_join(Path::new("/tmp"), "x");
    let _: Result<Option<PathBuf>, adr_fmt::ContainmentError> =
        adr_fmt::contained_join_optional(Path::new("/tmp"), "x");

    let parse_domain_fn: fn(
        &adr_fmt::DomainDir,
    ) -> Result<adr_fmt::ParseOutcome, adr_fmt::ParseError> = adr_fmt::parse_domain;
    assert!(std::ptr::fn_addr_eq(
        parse_domain_fn,
        adr_fmt::parse_domain as fn(_) -> _
    ));
    let parse_stale_fn: fn(
        &Path,
        &adr_fmt::Config,
    ) -> Result<adr_fmt::ParseOutcome, adr_fmt::ParseError> = adr_fmt::parse_stale;
    assert!(std::ptr::fn_addr_eq(
        parse_stale_fn,
        adr_fmt::parse_stale as fn(_, _) -> _,
    ));

    let load_quiet_fn: fn(&Path) -> Result<adr_fmt::Config, adr_fmt::LoadError> =
        adr_fmt::load_quiet;
    assert!(std::ptr::fn_addr_eq(
        load_quiet_fn,
        adr_fmt::load_quiet as fn(_) -> _,
    ));
    let _ = adr_fmt::resolve_corpus_root;

    let _: fn() -> Vec<adr_fmt::AdrRecord> = || Vec::new();
}

#[test]
fn adr_id_error_reachable_and_matchable_at_crate_root() {
    let err = adr_fmt::AdrId::try_new("x", 0).unwrap_err();
    let named: adr_fmt::AdrIdError = err;
    match named {
        adr_fmt::AdrIdError::PrefixLength { .. }
        | adr_fmt::AdrIdError::PrefixNotUppercaseAscii { .. }
        | adr_fmt::AdrIdError::NumberOutOfRange { .. }
        | adr_fmt::AdrIdError::Malformed { .. } => {}
        _ => panic!("unexpected AdrIdError variant"),
    }
}
