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

echo "==> All adr-fmt verification checks passed."
