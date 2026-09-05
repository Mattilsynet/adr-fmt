//! Smoke + API probe tests pinning the lib surface.
//!
//! `run_default_mode_via_lib_api_returns_zero` proves `adr_fmt::run` is
//! callable from a library consumer. `lib_api_modules_resolve` is a
//! compile-time probe that every item in the Q2 public-API set (see bd
//! adr-fmt-d7ao) resolves under its re-exported crate-root path.
//!
//! The `--help` / `--version` termination guards run out-of-process: an
//! in-process assertion cannot bite, because `process::exit(0)` inside
//! `run` would terminate the test binary *successfully* before the
//! assertion executes. Each parent test spawns an `#[ignore]`d child
//! probe in this same executable and requires a sentinel printed *after*
//! `run` returns; a terminating `run` yields a successful child with no
//! sentinel, which fails the parent.
//!
//! Modules `context`, `nav`, `output`, `refs`, `rules`, `guidelines` are
//! private per CHE-0030 (Flat Public API via Private Modules); external
//! consumers MUST NOT name those paths, so no probes exist for them.
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
    let exit: i32 = adr_fmt::run(argv);
    println!("{SENTINEL} help {exit}");
}

#[test]
#[ignore = "spawned by the termination-guard parent test"]
fn child_probe_version() {
    let argv: Vec<OsString> = vec![OsString::from("adr-fmt"), OsString::from("--version")];
    let exit: i32 = adr_fmt::run(argv);
    println!("{SENTINEL} version {exit}");
}

#[test]
fn run_default_mode_via_lib_api_returns_zero() {
    let argv: Vec<OsString> = vec![OsString::from("adr-fmt")];
    let exit: i32 = adr_fmt::run(argv);
    assert_eq!(exit, 0, "default-mode run should exit 0");
}

#[test]
fn help_returns_to_caller_instead_of_terminating_the_process() {
    let stdout = spawn_child_probe("child_probe_help");
    assert!(
        stdout.contains(&format!("{SENTINEL} help 0")),
        "`run` must return control to the caller with exit 0 for --help \
         (AFM-0003:R1); the post-call sentinel was absent, which means the \
         process terminated inside `run`. child stdout:\n{stdout}"
    );
}

#[test]
fn version_returns_to_caller_instead_of_terminating_the_process() {
    let stdout = spawn_child_probe("child_probe_version");
    assert!(
        stdout.contains(&format!("{SENTINEL} version 0")),
        "`run` must return control to the caller with exit 0 for --version \
         (AFM-0003:R1); the post-call sentinel was absent, which means the \
         process terminated inside `run`. child stdout:\n{stdout}"
    );
}

#[test]
fn parse_error_returns_to_caller_instead_of_terminating_the_process() {
    let argv: Vec<OsString> = vec![
        OsString::from("adr-fmt"),
        OsString::from("--no-such-flag-exists"),
    ];
    let exit: i32 = adr_fmt::run(argv);
    assert_eq!(exit, 2, "an unknown flag is a clap usage error (exit 2)");
}

#[test]
fn mutually_exclusive_modes_return_a_conflict_error() {
    let argv: Vec<OsString> = vec![
        OsString::from("adr-fmt"),
        OsString::from("--lint"),
        OsString::from("--tree"),
    ];
    let exit: i32 = adr_fmt::run(argv);
    assert_eq!(exit, 2, "clap-declared exclusivity must still reject");
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
