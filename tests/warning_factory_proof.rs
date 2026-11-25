#[test]
fn warning_factory_proof_is_green() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(manifest.join(".ooda").join("tmp")).unwrap();
    let status = std::process::Command::new("python3")
        .arg(manifest.join("tests").join("warning_factory.py"))
        .status()
        .unwrap();
    assert!(
        status.success(),
        "the AFM-0038 bounded warning-details proof tests/warning_factory.py must \
         run green; python3 exited with {status}"
    );
}
