//! ADR template and link-integrity validator — binary entry-point.
//!
//! Thin wrapper around [`adr_fmt::run`]. All CLI parsing and dispatch
//! logic lives in the library crate so future consumers (e.g.
//! `adr-srv`, Phase 2 v2 C1) can re-use the surface without spawning
//! a subprocess. CLI behaviour is frozen for v0.1 per AFM-0036:R2.
//!
//! This is the only authorised process-exit site per AFM-0026:R4, and
//! the only site mapping [`adr_fmt::RunError`] onto the exit codes
//! AFM-0003:R1 fixes. The library renders its own messages and returns
//! a typed result rather than terminating.

#![forbid(unsafe_code)]

use adr_fmt::RunError;

fn main() {
    std::process::exit(match adr_fmt::run(std::env::args_os()) {
        Ok(()) => 0,
        Err(RunError::Infrastructure) => 1,
        Err(RunError::Usage) => 2,
    });
}
