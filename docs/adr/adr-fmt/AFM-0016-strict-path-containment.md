# AFM-0016. Strict Path Containment for Config-Supplied Directories

Date: 2026-04-29
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

References: AFM-0003

## Context

Configured corpus, domain and stale paths must not escape their containing
root through absolute paths, parent traversal or symlinks. AFM-0003:R1 is
the constraining parent because containment failure uses the infrastructure
channel rather than advisory approval. Lexical checks alone cannot detect
symlink escape. This policy addresses untrusted configuration, not a peer
process mutating paths concurrently with reads.

## Decision

Reject any config-supplied directory string that fails strict
containment, treating violations as infrastructure errors per
AFM-0003:R1.

R1 [5]: Validate every config-supplied directory through
  `containment::contained_join` or `contained_join_optional` in
  `src/containment.rs`, which enforce the
  full lexical-plus-canonical pipeline
R2 [5]: Reject lexically as `ContainmentError::Absolute`,
  `ContainmentError::Empty`, or `ContainmentError::ParentTraversal`
  any segment that is absolute, empty, or contains a
  `Component::ParentDir` before touching the filesystem
R3 [5]: Canonicalize the joined target via `std::fs::canonicalize`
  and verify it descends from the canonicalized ADR root via
  `Path::starts_with`; reject mismatches as
  `ContainmentError::EscapesRoot`
R4 [5]: Surface containment failures on stderr in the library and return
   an infrastructure error for `src/main.rs` to map to exit 1,
   preserving the AFM-0003:R1 infrastructure-error channel
R5 [5]: Canonicalize the selected walk-up marker directory and resolve
   its configured corpus root through `resolve_corpus_root` so subsequent
   containment checks use a symlink-resolved root without a CLI path override

## Consequences

- Easier: one lexical/canonical pipeline rejects escaping configured paths.
- Harder: shared archives cannot be joined through out-of-tree symlinks.
- Risks: validation is not race-free containment; concurrent filesystem mutation
  remains outside the threat model. Optional absence differs from probe failure.

Source evidence: `src/containment.rs:111–198` validates joins;
`src/config.rs:252–268` resolves the corpus root;
`src/lib.rs:568–591` canonicalizes the marker and discovers domains.
These source checks do not demonstrate resistance to concurrent attackers or
establish a filesystem deadline. Unsuitable-marker fallback remains governed
by AFM-0001:R8, not silently strengthened here.
