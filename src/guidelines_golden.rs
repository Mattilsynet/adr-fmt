//! Byte-level golden pin on the two default-mode governance renderings.
//!
//! `tests/guidelines_parity.rs` gates rule-ID parity only: an N002/N003
//! description swap, or wrong wording on any of the rules outside the
//! N-family keyword anchors, passes it green. This pin compares the
//! rendered bytes against a committed file, so any wording change becomes
//! a reviewable diff instead of silent drift.
//!
//! `governance.txt` is rendered against this repository's own
//! `adr-fmt.toml`; `setup_guide.txt` takes no config. Rendering happens
//! in-process into a `Vec<u8>` — the same bytes the binary writes to
//! stdout, since `run_default_mode` calls these functions directly.
//!
//! # Regeneration
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test        # rewrites the goldens, then FAILS
//! cargo test                        # verifies; this is the run that may pass
//! ```
//!
//! Regeneration is deliberately not a passing path. `UPDATE_GOLDEN=1`
//! rewrites both golden files from the current renderer and then fails the
//! run, so no single command can both mutate the expected output and report
//! success; verification always requires a second run with the variable
//! unset. Any value other than exactly `1` is rejected rather than treated
//! as "on", so an inherited `UPDATE_GOLDEN=0` cannot quietly disarm the
//! pin. Review the resulting diff: it is the whole point of the pin.
//!
//! There is no CLI flag for regeneration — the binary's CLI surface is
//! frozen for v0.1 by AFM-0001:R1.

use std::fs;
use std::path::PathBuf;

use crate::config::load_quiet;
use crate::guidelines::{print_governance, print_setup_guide};

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name)
}

enum GoldenMode {
    Verify,
    Regenerate,
}

fn golden_mode() -> GoldenMode {
    match std::env::var_os("UPDATE_GOLDEN") {
        None => GoldenMode::Verify,
        Some(value) if value == "1" => GoldenMode::Regenerate,
        Some(value) => panic!(
            "UPDATE_GOLDEN must be exactly `1` to request regeneration; found {value:?}. \
             Unset it to verify against the committed goldens — an inherited value must \
             never decide whether this guard runs"
        ),
    }
}

fn first_difference(expected: &[u8], actual: &[u8]) -> String {
    let offset = expected
        .iter()
        .zip(actual)
        .position(|(want, got)| want != got)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    let located = format!(
        "first differing byte at offset {offset} (golden {} bytes, actual {} bytes)",
        expected.len(),
        actual.len()
    );
    let expected = String::from_utf8_lossy(expected);
    let actual = String::from_utf8_lossy(actual);
    for (index, (want, got)) in expected.lines().zip(actual.lines()).enumerate() {
        if want != got {
            return format!(
                "{located}\nline {}:\n  golden: {want:?}\n  actual: {got:?}",
                index + 1
            );
        }
    }
    format!(
        "{located}\nline counts differ: golden has {} line(s), actual has {} line(s)",
        expected.lines().count(),
        actual.lines().count()
    )
}

fn regenerate_golden(name: &str, rendered: &[u8]) -> ! {
    let path = golden_path(name);
    let parent = path.parent().expect("golden path has a parent directory");
    fs::create_dir_all(parent).expect("golden directory is creatable");
    fs::write(&path, rendered).expect("golden file is writable");
    panic!(
        "{}: golden rewritten from the current renderer. This run fails by design so that \
         regeneration can never also report PASS. Review the diff, then re-run `cargo test` \
         with UPDATE_GOLDEN unset to verify",
        path.display()
    )
}

fn assert_golden(name: &str, rendered: &[u8]) {
    match golden_mode() {
        GoldenMode::Regenerate => regenerate_golden(name, rendered),
        GoldenMode::Verify => (),
    }
    let path = golden_path(name);
    let expected = fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "{}: golden file unreadable ({e}); regenerate with `UPDATE_GOLDEN=1 cargo test`",
            path.display()
        )
    });
    assert!(
        expected == rendered,
        "{}: rendered governance output no longer matches its golden pin. If the change is \
         intended, regenerate with `UPDATE_GOLDEN=1 cargo test` and review the diff; if it is \
         not, the renderer regressed.\n{}",
        path.display(),
        first_difference(&expected, rendered)
    );
}

#[test]
fn governance_output_matches_golden() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = load_quiet(&manifest).expect("this repository carries a usable adr-fmt.toml");
    let mut rendered = Vec::new();
    print_governance(&mut rendered, &config).expect("writing into a Vec cannot fail");
    assert_golden("governance.txt", &rendered);
}

#[test]
fn setup_guide_matches_golden() {
    let mut rendered = Vec::new();
    print_setup_guide(&mut rendered).expect("writing into a Vec cannot fail");
    assert_golden("setup_guide.txt", &rendered);
}
