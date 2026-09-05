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
#
# Exit 0 = at or under threshold. Exit 1 = over threshold.
# Exit 2 = the gate could not obtain a verdict (lint failed, or the
# header was absent or unparseable) — never conflated with "clean".

set -euo pipefail

threshold="${ADR_LINT_MAX_WARNINGS:-8}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

if ! output="$(cargo run -q --locked -- --lint 2>&1)"; then
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

if [ "$header" -gt "$threshold" ]; then
    printf '%s\n' "$output" >&2
    echo "adr-lint-gate: $header warning(s) exceeds threshold $threshold" >&2
    exit 1
fi

echo "adr-lint-gate: $header warning(s), threshold $threshold — ok"
