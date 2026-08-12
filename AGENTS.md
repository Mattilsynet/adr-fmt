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

This tool's own design decisions are tracked upstream in
`Mattilsynet/gh-report` under `docs/adr/adr-fmt/` (prefix `AFM`); this
repo does not yet carry its own ADR corpus.
