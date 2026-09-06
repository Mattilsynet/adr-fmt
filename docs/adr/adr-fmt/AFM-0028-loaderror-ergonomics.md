# AFM-0028. LoadError Ergonomics Amendment to AFM-0026

Date: 2026-05-18
Last-reviewed: 2026-09-06
Tier: S
Status: Accepted

## Related

References: AFM-0026, CHE-0030, COM-0013

## Context

AFM-0026:R1 defines the public error set and is the first parent: without
that exported surface, this trait floor has no governed subjects.
Consumers need standard error formatting and propagation without per-variant
shims. `LoadError` implements Debug, Display and `std::error::Error` in
`src/config.rs:186–220`; this is a maintained contract, not pending work.

The trait floor applies to future exported errors as well as existing ones.
AFM-0036:R2–R4 governs compatibility, and AFM-0029:R2 permits narrow in-place
amendments. Keeping a distinct trait-floor decision makes this obligation
directly citable without claiming that Accepted ADRs are immutable.

## Decision

Amend AFM-0026 by adding a trait-surface constraint on every public
error type in the AFM-0026:R1 set; reaffirm semver stability of those
trait surfaces under AFM-0026:R3; and make the constraint inheritable
so future error types added to the R1 surface do not each need their
own follow-up ADR.

R1 [5]: Every public error type in the AFM-0026:R1 surface set MUST
   implement `core::fmt::Display`, `core::fmt::Debug`, and
   `std::error::Error`, including future additions to that public set.

R2 [4]: Each AFM-0026:R1 public error's `Display` impl MUST produce a human-readable,
  single-line-preferred message. The impl MUST NOT include sensitive
  paths or values beyond what the variant already names in its
  public-field contract per AFM-0026:R3. Existing variant fields (e.g.
  file paths inside `LoadError::Io(String)`) are already part of the
  public surface and may appear unchanged in the Display output.

R3 [5]: The `Display`, `Debug`, and `std::error::Error` trait surfaces
  of AFM-0026:R1 error types are part of the v0.1 semver contract per
  the extension of AFM-0026:R3. New trait impls may be added in minor
  versions; existing trait impls MUST NOT be removed or reshaped.
  Field-shape stability of error variants themselves is already
  governed by AFM-0026:R3 and remains unchanged.

R4 [4]: Future error types added to the AFM-0026:R1 surface —
  including any type added under AFM-0026:R5's "ADR with
  current-consumer justification" clause — inherit R1 by construction.
  No per-type follow-up ADR is required to establish
  Display/Debug/std::error::Error coverage. A successor ADR is
  required only to RELAX this rule.

## Consequences

+ becomes easier: consumers use standard error formatting and propagation
  rather than library-specific trait shims.
− becomes harder: every exported error needs three maintained trait impls
  and compatibility review when their behavior changes.
risks/migration: this documentation repair changes no error fields or traits;
  downstream shim removal is neither required nor verified by this checkpoint.

Evidence: `src/config.rs:186–220` provides the implemented LoadError floor;
`tests/lib_smoke.rs` contains external API probes. The execution record is
under `adr-fmt-rjo8.3`; no external `adr-srv` build is claimed.
