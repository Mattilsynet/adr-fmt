//! ADR template and link-integrity validator — binary entry-point.
//!
//! Wraps [`adr_fmt::run`]; library-owned CLI parsing and dispatch let
//! consumers such as `adr-srv` avoid subprocesses. CLI compatibility belongs
//! to the v0.1 contract per AFM-0036:R2 and AFM-0026:R5.
//!
//! Sole authorised process-exit site per AFM-0026:R4, mapping
//! [`adr_fmt::RunError`] to AFM-0003:R1 exit codes. The library renders
//! messages and returns a typed result rather than terminating.

#![forbid(unsafe_code)]

use adr_fmt::RunError;

fn main() {
    std::process::exit(match adr_fmt::run(std::env::args_os()) {
        Ok(()) => 0,
        Err(RunError::Infrastructure) => 1,
        Err(RunError::Usage) => 2,
    });
}
