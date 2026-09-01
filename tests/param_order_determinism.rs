//! Multi-process falsifier for PARAMETER OVERRIDES ordering.
//!
//! `HashMap` iteration order is randomised per process but fixed *within* a
//! process, so an in-process repetition test cannot observe the defect. This
//! test spawns the binary 20 separate times and compares raw stdout bytes.

use std::fs;
use std::process::Command;

use tempfile::TempDir;

const SPAWNS: usize = 20;

const CONFIG: &str = r#"
[corpus]
root = "docs/adr"

[stale]
directory = "stale"

[[domains]]
prefix = "TST"
name = "Test Domain"
directory = "test"
description = "Determinism test domain."
crates = ["test-core"]

[[rules]]
id = "T015"
params = { min_words = 7, max_words = 120 }

[[rules]]
id = "T016"
params = { max_rules = 10, min_rule_words = 7, max_rule_words = 60 }
"#;

fn corpus() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::create_dir_all(dir.path().join("docs/adr/test")).expect("create domain dir");
    fs::write(dir.path().join("adr-fmt.toml"), CONFIG).expect("write config");
    dir
}

fn run_default_mode(dir: &TempDir) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_adr-fmt"))
        .current_dir(dir.path())
        .output()
        .expect("spawn adr-fmt");
    assert!(
        output.status.success(),
        "adr-fmt exited {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn parameter_overrides_are_byte_identical_across_twenty_processes() {
    let dir = corpus();

    let first = run_default_mode(&dir);
    for spawn in 1..SPAWNS {
        let next = run_default_mode(&dir);
        assert!(
            next == first,
            "stdout differed between process 0 and process {spawn}"
        );
    }

    let rendered = String::from_utf8(first).expect("stdout is utf-8");
    assert!(
        rendered.contains("  T015  max_words=120, min_words=7\n"),
        "expected alphabetical T015 override row, got:\n{rendered}"
    );
    assert!(
        rendered.contains("  T016  max_rule_words=60, max_rules=10, min_rule_words=7"),
        "expected alphabetical T016 override row, got:\n{rendered}"
    );
}
