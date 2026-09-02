//! ADR template and link-integrity validator — binary entry-point.
//!
//! Thin wrapper around [`adr_fmt::run`]. All CLI parsing and dispatch
//! logic lives in the library crate so future consumers (e.g.
//! `adr-srv`, Phase 2 v2 C1) can re-use the surface without spawning
//! a subprocess. CLI behaviour is frozen for v0.1 per AFM-0001.
//!
//! This is the only authorised process-exit site per AFM-0026:R4. The
//! library returns clap's parse outcome rather than terminating, and the
//! mapping from that outcome to an exit code happens here: clap renders
//! its own message and supplies the code, which is `0` for `--help` and
//! `--version` and non-zero for a genuine parse failure (AFM-0003:R1).

#![forbid(unsafe_code)]

fn main() {
    let code = match adr_fmt::run(std::env::args_os()) {
        Ok(code) => code,
        Err(err) => {
            let _ = err.print();
            err.exit_code()
        }
    };
    std::process::exit(code);
}
