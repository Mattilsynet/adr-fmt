#!/usr/bin/env bash
#
# Executable contract tests for scripts/adr-lint-gate.sh.
#
# The gate is enforcement tooling: a false-clean here disables
# AFM-0003:R3 while CI stays green. These tests pin the verdict
# contract (0 = at/under, 1 = over, 2 = no verdict) against a stub
# lint producer injected via ADR_LINT_CMD, so parsing and exit codes
# are exercised without a cargo build.
#
# Usage: scripts/adr-lint-gate-test.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gate="$repo_root/scripts/adr-lint-gate.sh"

failures=0
cases=0

expect() {
    local name="$1" want="$2" cmd="$3" threshold="$4"
    local got=0
    cases=$((cases + 1))
    if [ -n "$threshold" ]; then
        ADR_LINT_CMD="$cmd" ADR_LINT_MAX_WARNINGS="$threshold" bash "$gate" >/dev/null 2>&1 || got=$?
    else
        ADR_LINT_CMD="$cmd" bash "$gate" >/dev/null 2>&1 || got=$?
    fi
    if [ "$got" -eq "$want" ]; then
        printf 'ok   %s (exit %s)\n' "$name" "$got"
    else
        printf 'FAIL %s: want exit %s, got %s\n' "$name" "$want" "$got"
        failures=$((failures + 1))
    fi
}

clean_output='## Diagnostics: 8 warning(s) across 32 ADR(s)'
over_output='## Diagnostics: 9 warning(s) across 32 ADR(s)'
no_header_output='some other output entirely'

expect "at threshold is clean"        0 "printf '%s\n' '$clean_output'"     "8"
expect "under threshold is clean"     0 "printf '%s\n' '$clean_output'"     "9"
expect "over threshold fails"         1 "printf '%s\n' '$over_output'"      "8"
expect "default threshold applies"    0 "printf '%s\n' '$clean_output'"     ""
expect "missing header is no verdict" 2 "printf '%s\n' '$no_header_output'" "8"
expect "producer failure no verdict"  2 "exit 3"                            "8"
expect "malformed threshold"          2 "printf '%s\n' '$clean_output'"     "bogus"
expect "negative threshold"           2 "printf '%s\n' '$clean_output'"     "-1"
expect "empty threshold"              2 "printf '%s\n' '$clean_output'"     " "
expect "float threshold"              2 "printf '%s\n' '$clean_output'"     "8.5"

printf '\nadr-lint-gate-test: %s case(s), %s failure(s)\n' "$cases" "$failures"
[ "$failures" -eq 0 ]
