#!/usr/bin/env sh
# Closed-error enforcement is in-repo: tests/closed_error_policy.rs parses src/
# with syn. There is no external gate binary to probe or fall back to.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "==> Running closed-error ratchet check (in-repo)..."
cargo test --locked --test closed_error_policy

echo "==> Running unsafe-code gate check (forbid-unsafe-total / RST-0005)..."
# All crate roots must enforce #![forbid(unsafe_code)]
grep -q 'forbid(unsafe_code)' "$ROOT/src/lib.rs" || {
  echo "::error::forbid-unsafe-total: src/lib.rs lacks #![forbid(unsafe_code)] (RST-0005)" >&2
  exit 1
}
grep -q 'forbid(unsafe_code)' "$ROOT/src/main.rs" || {
  echo "::error::forbid-unsafe-total: src/main.rs lacks #![forbid(unsafe_code)] (RST-0005)" >&2
  exit 1
}

echo "==> Running cargo test..."
cargo test --locked

echo "==> Running cargo clippy..."
cargo clippy --all-targets --locked -- -D warnings

echo "==> Running cargo fmt check..."
cargo fmt --all -- --check

echo "==> Running cargo doc (deny warnings)..."
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked

# comment-free is not a workspace member; obtain it via the pinned-install
# model (canonical git, rev v0.3.0) so local == CI. Without the pin a stale
# 0.2.0 binary on PATH would diverge from CI's freshly installed v0.3.0.
ensure_comment_free() {
  if command -v comment-free >/dev/null 2>&1 \
    && [ "$(comment-free --version 2>/dev/null | awk '{print $2}')" = "0.3.0" ]; then
    return 0
  fi
  echo "==> Installing comment-free v0.3.0 from canonical git (rev v0.3.0, --locked)..."
  cargo +1.100.0 install --git https://github.com/acje/comment-free --rev v0.3.0 --locked comment-free
}
ensure_comment_free

echo "==> Running comment-free doc-budget gate (80 advisory / 120 enforced)..."
comment-free --check-doc-budget --doc-advisory-words 80 --doc-max-words 120 --max-warning-files 0 .

echo "==> Running comment-free rewrite dry-run..."
comment-free --rewrite --dry-run .

echo "==> All adr-fmt verification checks passed."
