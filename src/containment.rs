//! Strict path containment for config-supplied directory strings.
//!
//! [`contained_join`] validates lexically (reject absolute paths and
//! `..` traversal) then canonically (reject symlink escapes outside
//! the ADR root), per AFM-0016 R1–R3. Failures surface as
//! [`ContainmentError`] with a user-facing reason.

use std::fmt;
use std::path::{Component, Path, PathBuf};

/// Reason a path was rejected by [`contained_join`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContainmentError {
    /// Segment is an absolute path.
    Absolute(String),
    /// Segment contains a `..` component (parent traversal).
    ParentTraversal(String),
    /// Segment is empty.
    Empty,
    /// Canonicalization of the ADR root failed while resolving
    /// `segment`; the root itself is unusable.
    RootCanonicalizeFailed {
        segment: String,
        kind: std::io::ErrorKind,
    },
    /// Canonicalization of the joined target failed.
    TargetCanonicalizeFailed {
        segment: String,
        kind: std::io::ErrorKind,
    },
    /// Probing the joined path for existence failed for a reason
    /// other than absence (permission denied, IO error): whether the
    /// path exists is indeterminate.
    MetadataFailed { segment: String, reason: String },
    /// Probing the joined path for existence failed for a reason
    /// other than absence; `kind` distinguishes permission failure
    /// from transient I/O error. Whether the path exists is
    /// indeterminate.
    MetadataProbeFailed {
        segment: String,
        kind: std::io::ErrorKind,
    },
    /// Canonical target escapes the canonical root via symlink or
    /// otherwise resolves outside the ADR corpus.
    EscapesRoot {
        segment: String,
        canonical_target: PathBuf,
        canonical_root: PathBuf,
    },
}

impl fmt::Display for ContainmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute(s) => write!(
                f,
                "path {} is absolute; config directories must be relative to the ADR root",
                s.escape_debug()
            ),
            Self::ParentTraversal(s) => write!(
                f,
                "path {} contains a parent-traversal component (..); config directories must stay within the ADR root",
                s.escape_debug()
            ),
            Self::Empty => write!(f, "path segment is empty"),
            Self::RootCanonicalizeFailed { segment, kind } => {
                write!(
                    f,
                    "cannot canonicalize the ADR root while resolving {}: {kind}",
                    segment.escape_debug()
                )
            }
            Self::TargetCanonicalizeFailed { segment, kind } => {
                write!(f, "cannot canonicalize {}: {kind}", segment.escape_debug())
            }
            Self::MetadataFailed { segment, reason } => {
                write!(
                    f,
                    "cannot determine whether {} exists: {reason}",
                    segment.escape_debug()
                )
            }
            Self::MetadataProbeFailed { segment, kind } => {
                write!(
                    f,
                    "cannot determine whether {} exists: {kind}",
                    segment.escape_debug()
                )
            }
            Self::EscapesRoot {
                segment,
                canonical_target,
                canonical_root,
            } => write!(
                f,
                "path {} resolves to {} which escapes the ADR root {} (likely via symlink)",
                segment.escape_debug(),
                canonical_target.display(),
                canonical_root.display()
            ),
        }
    }
}

impl std::error::Error for ContainmentError {}

/// Join `segment` to `root` after enforcing strict containment.
///
/// Returns the canonicalized target path on success. The target
/// must exist on disk (`std::fs::canonicalize` requires existence
/// on every supported platform); for paths that may legitimately
/// be absent at runtime, use [`contained_join_optional`] instead
/// of pre-checking with `Path::exists` (which would race the
/// canonicalize call).
///
/// # Errors
///
/// Returns [`ContainmentError`] when `segment` is empty, absolute,
/// contains parent traversal, cannot be canonicalized, or resolves
/// outside `root`.
pub fn contained_join(root: &Path, segment: &str) -> Result<PathBuf, ContainmentError> {
    lexical_check(segment)?;

    let joined = root.join(segment);
    let canonical_root =
        std::fs::canonicalize(root).map_err(|e| ContainmentError::RootCanonicalizeFailed {
            segment: segment.to_owned(),
            kind: e.kind(),
        })?;
    let canonical_target =
        std::fs::canonicalize(&joined).map_err(|e| ContainmentError::TargetCanonicalizeFailed {
            segment: segment.to_owned(),
            kind: e.kind(),
        })?;

    if !canonical_target.starts_with(&canonical_root) {
        return Err(ContainmentError::EscapesRoot {
            segment: segment.to_owned(),
            canonical_target,
            canonical_root,
        });
    }

    Ok(canonical_target)
}

/// Join `segment` to `root` after lexical checks; canonicalize
/// only if the target exists. Returns `Ok(None)` only when the
/// existence probe fails with [`std::io::ErrorKind::NotFound`] —
/// absence is never inferred from any other IO failure.
///
/// The probe does not follow symlinks, so a dangling symlink is a
/// present-but-unresolvable entry ([`ContainmentError::TargetCanonicalizeFailed`]),
/// not an absent one.
///
/// Used for paths that are optional at runtime (e.g., the stale
/// directory may not exist in a fresh repo).
///
/// # Errors
///
/// Returns [`ContainmentError`] when `segment` is empty, absolute,
/// contains parent traversal, cannot be probed for existence,
/// cannot be canonicalized, or resolves outside `root`.
pub fn contained_join_optional(
    root: &Path,
    segment: &str,
) -> Result<Option<PathBuf>, ContainmentError> {
    lexical_check(segment)?;

    let joined = root.join(segment);
    match std::fs::symlink_metadata(&joined) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(ContainmentError::MetadataProbeFailed {
                segment: segment.to_owned(),
                kind: e.kind(),
            });
        }
    }

    contained_join(root, segment).map(Some)
}

fn lexical_check(segment: &str) -> Result<(), ContainmentError> {
    if segment.is_empty() {
        return Err(ContainmentError::Empty);
    }

    let path = Path::new(segment);

    if path.is_absolute() {
        return Err(ContainmentError::Absolute(segment.to_owned()));
    }

    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(ContainmentError::ParentTraversal(segment.to_owned()));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ContainmentError::Absolute(segment.to_owned()));
            }
            Component::Normal(_) | Component::CurDir => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().expect("create tempdir")
    }

    #[test]
    fn rejects_absolute_path() {
        let dir = tmp();
        let err = contained_join(dir.path(), "/etc").unwrap_err();
        assert!(matches!(err, ContainmentError::Absolute(_)), "got: {err:?}");
    }

    #[test]
    fn rejects_parent_traversal() {
        let dir = tmp();
        let err = contained_join(dir.path(), "../etc").unwrap_err();
        assert!(
            matches!(err, ContainmentError::ParentTraversal(_)),
            "got: {err:?}"
        );
    }

    #[test]
    fn rejects_parent_traversal_mid_path() {
        let dir = tmp();
        let err = contained_join(dir.path(), "domain/../../etc").unwrap_err();
        assert!(
            matches!(err, ContainmentError::ParentTraversal(_)),
            "got: {err:?}"
        );
    }

    #[test]
    fn rejects_empty_segment() {
        let dir = tmp();
        let err = contained_join(dir.path(), "").unwrap_err();
        assert!(matches!(err, ContainmentError::Empty), "got: {err:?}");
    }

    #[test]
    fn accepts_normal_subdirectory() {
        let dir = tmp();
        let sub = dir.path().join("cherry");
        fs::create_dir(&sub).unwrap();
        let result = contained_join(dir.path(), "cherry").unwrap();
        assert!(result.starts_with(fs::canonicalize(dir.path()).unwrap()));
        assert!(result.ends_with("cherry"));
    }

    #[test]
    fn accepts_nested_subdirectory() {
        let dir = tmp();
        fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        let result = contained_join(dir.path(), "a/b/c").unwrap();
        assert!(result.ends_with("a/b/c"));
    }

    #[test]
    fn rejects_canonicalize_missing_target() {
        let dir = tmp();
        let err = contained_join(dir.path(), "does-not-exist").unwrap_err();
        assert!(
            matches!(err, ContainmentError::TargetCanonicalizeFailed { .. }),
            "got: {err:?}"
        );
    }

    #[test]
    fn optional_join_returns_none_for_missing() {
        let dir = tmp();
        let result = contained_join_optional(dir.path(), "missing").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn optional_join_still_rejects_absolute() {
        let dir = tmp();
        let err = contained_join_optional(dir.path(), "/etc").unwrap_err();
        assert!(matches!(err, ContainmentError::Absolute(_)), "got: {err:?}");
    }

    #[cfg(unix)]
    #[test]
    fn optional_join_dangling_symlink_is_not_absent() {
        use std::os::unix::fs::symlink;

        let dir = tmp();
        symlink(dir.path().join("no-such-target"), dir.path().join("link")).unwrap();

        let err = contained_join_optional(dir.path(), "link")
            .expect_err("dangling symlink must not be reported as absent");
        assert!(
            matches!(err, ContainmentError::TargetCanonicalizeFailed { .. }),
            "got: {err:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let parent = tmp();
        let outside = parent.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let root = parent.path().join("root");
        fs::create_dir(&root).unwrap();
        symlink(&outside, root.join("escape")).unwrap();

        let err = contained_join(&root, "escape").unwrap_err();
        assert!(
            matches!(err, ContainmentError::EscapesRoot { .. }),
            "got: {err:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn accepts_symlink_inside_root() {
        use std::os::unix::fs::symlink;

        let root = tmp();
        let real = root.path().join("real");
        fs::create_dir(&real).unwrap();
        symlink(&real, root.path().join("link")).unwrap();

        let result = contained_join(root.path(), "link").unwrap();
        assert!(result.starts_with(fs::canonicalize(root.path()).unwrap()));
    }

    #[test]
    fn root_and_target_canonicalize_failures_are_distinct_variants() {
        let dir = tmp();
        let missing_root = dir.path().join("no-such-root");

        let target_err = contained_join(dir.path(), "does-not-exist").unwrap_err();
        let root_err = contained_join(&missing_root, "anything").unwrap_err();

        assert!(
            matches!(
                target_err,
                ContainmentError::TargetCanonicalizeFailed {
                    kind: std::io::ErrorKind::NotFound,
                    ..
                }
            ),
            "got: {target_err:?}"
        );
        assert!(
            matches!(
                root_err,
                ContainmentError::RootCanonicalizeFailed {
                    kind: std::io::ErrorKind::NotFound,
                    ..
                }
            ),
            "got: {root_err:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_is_distinguishable_from_not_found() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tmp();
        let locked = dir.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::create_dir(locked.join("inner")).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let result = contained_join(dir.path(), "locked/inner");

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

        let err = result.expect_err("unreadable parent must not canonicalize");
        let ContainmentError::TargetCanonicalizeFailed { kind, .. } = err else {
            panic!("got: {err:?}");
        };
        assert_eq!(kind, std::io::ErrorKind::PermissionDenied);
        assert_ne!(kind, std::io::ErrorKind::NotFound);
    }

    #[test]
    fn canonicalize_failure_display_does_not_leak_absolute_paths() {
        let dir = tmp();
        let missing_root = dir.path().join("no-such-root");

        let root_err = contained_join(&missing_root, "anything").unwrap_err();
        let target_err = contained_join(dir.path(), "does-not-exist").unwrap_err();

        let root_text = root_err.to_string();
        let target_text = target_err.to_string();
        let leaked = dir.path().to_string_lossy().into_owned();

        assert!(!root_text.contains(&leaked), "leaked: {root_text}");
        assert!(!target_text.contains(&leaked), "leaked: {target_text}");
    }

    #[cfg(unix)]
    #[test]
    fn metadata_probe_permission_denied_is_distinguishable_from_not_found() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tmp();
        let locked = dir.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::create_dir(locked.join("inner")).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let result = contained_join_optional(dir.path(), "locked/inner");

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

        let err = result.expect_err("permission failure must not be reported as absent");
        let ContainmentError::MetadataProbeFailed { kind, .. } = err else {
            panic!("got: {err:?}");
        };
        assert_eq!(kind, std::io::ErrorKind::PermissionDenied);
        assert_ne!(kind, std::io::ErrorKind::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn metadata_probe_failure_display_does_not_leak_absolute_paths() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tmp();
        let locked = dir.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::create_dir(locked.join("inner")).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        let result = contained_join_optional(dir.path(), "locked/inner");

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();

        let text = result
            .expect_err("permission failure must not be reported as absent")
            .to_string();
        let leaked = dir.path().to_string_lossy().into_owned();

        assert!(!text.contains(&leaked), "leaked: {text}");
    }

    #[test]
    fn cur_dir_component_allowed() {
        let dir = tmp();
        fs::create_dir(dir.path().join("domain")).unwrap();
        let result = contained_join(dir.path(), "./domain").unwrap();
        assert!(result.ends_with("domain"));
    }
}
