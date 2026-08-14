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
- `rust-toolchain.toml` pins channel 1.97 (clippy + rustfmt). Use it,
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

`cargo test`/`clippy` above verify code, not corpus health. AFM-0003:R3
warning-threshold enforcement on the ADR corpus itself is unimplemented
— in CI and locally; a clean local build does not mean the corpus is
clean. Measured baseline: `adr-fmt --lint` reports `## Diagnostics: 25
warning(s) across 32 ADR(s)` and exits 0 by design.

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
