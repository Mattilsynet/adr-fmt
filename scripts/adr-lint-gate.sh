#!/usr/bin/env bash
#
# AFM-0003:R3 warning-threshold gate.
#
# `adr-fmt --lint` is advisory and exits 0 on findings by design
# (AFM-0003:R1/R2). This wrapper is the enforcement layer that lives
# outside the binary: it parses the `## Diagnostics: N warning(s)`
# header and fails when N exceeds the threshold.
#
# Usage:   scripts/adr-lint-gate.sh
# Tune:    ADR_LINT_MAX_WARNINGS=12 scripts/adr-lint-gate.sh
# Test:    ADR_LINT_CMD injects a stub lint producer; see
#          scripts/adr-lint-gate-test.sh for the pinned contract.
#
# Exit 0 = at or under threshold. Exit 1 = over threshold.
# Exit 2 = the gate could not obtain a verdict (lint failed, the
# header was absent or unparseable, or the threshold or the parsed
# warning count is not a bare decimal integer of 1 to 9 digits, i.e.
# 0 to 999999999) — never conflated with "clean".
#
# The 1-to-9-digit range is the contract, not an implementation
# detail: it is the widest range this script can compare with `[ -gt ]`
# without overflowing Bash's fixed-width arithmetic. An input outside
# it is rejected as no verdict rather than silently skipping the
# failure branch. Empty, non-numeric, negative, leading `+`, and
# whitespace values are rejected on the same footing.

set -euo pipefail

threshold="${ADR_LINT_MAX_WARNINGS-8}"
lint_cmd="${ADR_LINT_CMD:-cargo run -q --locked -- --lint}"

is_gate_integer() {
    [[ "$1" =~ ^[0-9]{1,9}$ ]]
}

if ! is_gate_integer "$threshold"; then
    echo "adr-lint-gate: ADR_LINT_MAX_WARNINGS='$threshold' is not a decimal integer of 1 to 9 digits (0-999999999); no verdict" >&2
    exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! output="$(eval "$lint_cmd" 2>&1)"; then
    printf '%s\n' "$output" >&2
    echo "adr-lint-gate: adr-fmt --lint failed to run; no verdict" >&2
    exit 2
fi

header="$(printf '%s\n' "$output" | sed -n 's/^## Diagnostics: \([0-9][0-9]*\) warning(s).*/\1/p' | head -n 1)"

if [ -z "$header" ]; then
    printf '%s\n' "$output" >&2
    echo "adr-lint-gate: no '## Diagnostics: N warning(s)' header found; no verdict" >&2
    exit 2
fi

if ! is_gate_integer "$header"; then
    printf '%s\n' "$output" >&2
    echo "adr-lint-gate: parsed warning count '$header' is not a decimal integer of 1 to 9 digits (0-999999999); no verdict" >&2
    exit 2
fi

if [ "$header" -gt "$threshold" ]; then
    printf '%s\n' "$output" >&2
    echo "adr-lint-gate: $header warning(s) exceeds threshold $threshold" >&2
    exit 1
fi

echo "adr-lint-gate: $header warning(s), threshold $threshold — ok"
