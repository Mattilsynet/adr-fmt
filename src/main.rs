//! ADR template and link-integrity validator — binary entry-point.
//!
//! Thin wrapper around [`adr_fmt::run`]. All CLI parsing and dispatch
//! logic lives in the library crate so future consumers (e.g.
//! `adr-srv`, Phase 2 v2 C1) can re-use the surface without spawning
//! a subprocess. CLI behaviour is frozen for v0.1 per AFM-0001.
//!
//! This is the only authorised process-exit site per AFM-0026:R4. The
//! library renders any CLI parse message itself and returns the exit
//! code rather than terminating; applying that code is this file's job.

#![forbid(unsafe_code)]

fn main() {
    std::process::exit(adr_fmt::run(std::env::args_os()));
}
