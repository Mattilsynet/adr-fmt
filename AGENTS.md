# AGENTS.md — adr-fmt

Repo-specific operational notes. General agent/OODA doctrine, bash hygiene,
and the Rust no-`//`-comments house style live in the global
`~/.config/opencode/AGENTS.md` — not repeated here. opencode is the only
agent surface for this repo.

## What this repo is

A single-crate Rust binary (+ library `adr_fmt`): a read-only ADR
template and link-integrity validator. It discovers `adr-fmt.toml` by
walking up from cwd, then lints/inspects an ADR corpus. See `README.md`
for modes, bootstrap, and usage.

## Build / test / lint

```
cargo build
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

- `clippy::pedantic` is the standing bar (`[lints.clippy] pedantic = warn`
  in `Cargo.toml`), not an elevation — new code passes it with zero
  warnings.
- `rustfmt` runs on **stable defaults only** — do not add a
  `rustfmt.toml` or `clippy.toml`.
- `rust-toolchain.toml` pins channel 1.98 (clippy + rustfmt). Use it,
  don't bump without cause.
- `cargo deny check` and `cargo audit` are supply-chain gates; run
  before publishing or bumping dependencies.

## Delivery

`main` is protected (PR required). Changes land via a feature branch and
a pull request — never a direct push to `main`.

## Governance

This repo carries its own ADR corpus at `docs/adr/adr-fmt/` (prefix
`AFM`, 22 Accepted). These ADRs are binding — read the relevant ones
before changing behaviour they govern. Read them with `adr-fmt --tree`
/ `adr-fmt --context <prefix>` or directly as markdown; the config file
`adr-fmt.toml` lives at the **project root**, not inside
`docs/adr/adr-fmt/` — look there first, not next to the ADR files.

`cargo test`/`clippy` above verify code, not corpus health. Corpus
health has its own locally-runnable gate — run it before handing off
work that touches `docs/adr/`:

```
scripts/adr-lint-gate.sh
```

It discharges AFM-0003:R3: `adr-fmt --lint` is advisory and exits 0 on
findings by design, so the gate parses the `## Diagnostics: N
warning(s)` header and exits 1 when N exceeds the threshold (exit 2
when it cannot obtain a count — never conflated with clean). The
threshold defaults to the measured baseline of **8** and is overridable
via `ADR_LINT_MAX_WARNINGS`. CI runs the same script.

Both the threshold and the parsed warning count must be a bare decimal
integer of **1 to 9 digits** (0–999999999); that is the widest range
the shell can compare without overflowing its fixed-width arithmetic.
Anything else — empty, non-numeric, negative, leading `+`, whitespace,
or more than 9 digits — is **no verdict: exit 2**, never a pass. An
explicitly empty `ADR_LINT_MAX_WARNINGS=''` is a malformed value, not
"unset"; only a genuinely unset variable falls back to the default 8.

Measured baseline: `adr-fmt --lint` reports `## Diagnostics: 8
warning(s) across 32 ADR(s)` and exits 0.

## ADR writing style

An ADR is a projection of the current state, stating only what is true
now. Overwrite superseded text in place rather than annotating it — git
already holds the prior state.

Annotation marks the correction but never the thing corrected, so dead
text goes on reading as live. The test: would a reader of *only that
sentence* get the current state right?

| Instead of | Write |
|---|---|
| "R3 previously required Y; it now requires X" | "R3 requires X" |
| an erratum, changelog, or edit-history note | fix the text; the diff is the record |
| "alternatives considered and rejected: A, B, C" | nothing — unless a reason prevents re-proposal, then state it as a rule carrying that reason |

This governs prose **inside** a document. Corpus-level supersession is
unchanged and stays in the structured fields — `Status:` and
`Supersedes:`, which `adr-fmt --refs` reads — never as prose archaeology
in `## Context` or `## Decision`. Whether the corpus itself becomes
mutable rather than immutable-plus-supersession is a separate governance
question this rule does not decide.
