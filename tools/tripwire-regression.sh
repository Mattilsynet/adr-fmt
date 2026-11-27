#!/usr/bin/env sh
set -eu

# tripwire-regression: Four-step guard proof harness proving gates bite
# 1. plant a compiling violation -> 2. observe failure -> 3. revert -> 4. observe clean
#
# Adapted from comment-free/tools/tripwire-regression.sh to adr-fmt's in-repo
# closed-error gate (tests/closed_error_policy.rs). Only the manual
# `std::error::Error` route is planted: adr-fmt has no `thiserror` dependency,
# so the derive route is covered by the `derive_shape_is_flagged` unit test
# instead of a live planted derive enum. AFM-0037:R1-R2 proves the RST-0006
# closed-error gate bites and cannot fail open.

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

LIB="$ROOT/src/lib.rs"
BACKUP=$(mktemp "${TMPDIR:-/tmp}/adr-fmt-lib.XXXXXX")
cp -f "$LIB" "$BACKUP"

restore_lib() {
  if [ -f "$BACKUP" ]; then
    cp -f "$BACKUP" "$LIB"
  fi
}
trap 'restore_lib' 0 1 2 15

expect_status() {
  expected=$1
  shift
  label=$1
  shift
  set +e
  "$@"
  actual=$?
  set -e
  echo "==> [tripwire-regression] $label exited $actual (expected $expected)"
  if [ "$actual" -ne "$expected" ]; then
    echo "::error::tripwire-regression: $label exited $actual (expected $expected)" >&2
    exit 1
  fi
}

echo "==> [tripwire-regression] Testing the closed-error gate bite via the manual Error route..."

cat >> "$LIB" <<'PLANT'

#[derive(Debug)]
#[non_exhaustive]
pub enum PlantedAdrError {
    Planted,
}
impl std::fmt::Display for PlantedAdrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("planted")
    }
}
impl std::error::Error for PlantedAdrError {}
PLANT

expect_status 0 "plant builds (cargo build --all-targets --locked)" cargo build --all-targets --locked

echo "==> [tripwire-regression] verify.sh with plant"
set +e
output=$(sh scripts/verify.sh 2>&1)
status=$?
set -e
echo "==> [tripwire-regression] verify.sh with plant exited $status (expected non-zero)"
if [ "$status" -eq 0 ]; then
  echo "::error::tripwire-regression: gate failed to bite on the planted closed-error violation" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -q 'PlantedAdrError'; then
  printf '%s\n' "$output" >&2
  echo "::error::tripwire-regression: closed-error gate reported no planted PlantedAdrError occurrence" >&2
  exit 1
fi
echo "==> [tripwire-regression] observed PlantedAdrError in the gate failure output (verify.sh exited $status)"

restore_lib

expect_status 0 "verify.sh after revert" sh scripts/verify.sh

if ! cmp -s "$BACKUP" "$LIB"; then
  echo "::error::tripwire-regression: src/lib.rs was not restored byte-for-byte" >&2
  exit 1
fi
rm -f "$BACKUP"

echo "OK: tripwire-regression all guard proofs verified clean."
