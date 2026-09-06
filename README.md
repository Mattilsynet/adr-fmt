# adr-fmt

ADR template and link-integrity validator. A read-only analysis tool that
serves as the single source of truth for ADR governance rules across a
multi-domain ADR corpus.

## Modes

```text
adr-fmt                     # default: print governance guidelines
adr-fmt --lint              # lint all ADRs
adr-fmt --lint --max-warning-docs 5  # details for five offending documents
adr-fmt --refs <ADR_ID>     # inbound references (References + Supersedes)
adr-fmt --context <CRATE>   # decision rules for a crate
adr-fmt --tree [DOMAIN]     # domain tree overview
```

The corpus location is discovered by walking up from the current
working directory until an `adr-fmt.toml` with a valid `[corpus]`
table is found. Run from anywhere inside the workspace.

Exit codes:
- `0` — analysis complete (warnings only, or clean)
- `1` — infrastructure error or lint errors detected

Warnings are advisory by design (per AFM-0003): a corpus emitting
warnings still exits `0`. Exit `1` is reserved for infrastructure
failures and structural lint errors that prevent analysis. Treat
warnings as signal for review, not as build-breakers.

### Warning detail and totals (0.2.0)

`--lint` scans the corpus and shows all diagnostics for the first offending
document by native source-path order. `--max-warning-docs N` changes that
document limit; `0` hides document details, not findings. Clean files consume
no slots; malformed files do. Configuration and directory-level diagnostics
remain visible at every limit. Within each source, details sort by line,
rule ID and message.

`N` must be a bare unsigned decimal integer fitting the platform's `usize`.
Empty, missing, signed, whitespace-padded, nondecimal and overflowing values
are usage errors. The explicit option requires `--lint`.

The unchanged `## Diagnostics: N warning(s) across M ADR(s)` header counts
all public diagnostics, including hidden documents; `M` counts parsed ADRs,
not offending files. Non-clean output also includes `### Warning totals by
kind` with nonzero `- RULE: COUNT` lines sorted by rule ID. Internal findings
are excluded. Clean output is unchanged. The duplicate-ID short circuit
still skips rule checks and explicitly reports incomplete validation; its
totals cover only parser and duplicate-ID findings, not unexecuted checks.

Neither exit zero nor absent detail means clean. Use the header count and
`scripts/adr-lint-gate.sh` for threshold enforcement, never count detail
bullets. The cap limits documents, not output bytes or scan memory: globals,
messages and whole-corpus processing remain unbounded by `N`.

## Configuration

For an additive, opt-in engineering baseline, see the
[software-factory corpus](software-factory/README.md). Its nested configuration
is independent of this tool's own AFM governance.

`adr-fmt.toml` lives at the workspace root. It declares the corpus
location, domains, the stale folder, and crate-to-domain mapping.
See this repository's own `adr-fmt.toml` for a worked example.

## Bootstrap on a fresh corpus

Starting from an empty repository (no existing ADRs):

1. **Install.** `cargo +1.98.0 install --git https://github.com/Mattilsynet/adr-fmt --locked adr-fmt`.
   For development, build this checkout with `cargo build --locked` and
   invoke `target/debug/adr-fmt` directly.

2. **Pick an ADR root.** Conventional choice: `docs/adr/`.

3. **Write `adr-fmt.toml`** at the repository root. Minimum viable:

   ```toml
   [corpus]
   root = "docs/adr"

   [stale]
   directory = "stale"

   [[domains]]
   prefix = "ARC"          # 2–4 uppercase letters
   name = "Architecture"
   directory = "arc"       # relative to corpus.root
   description = "Cross-cutting architectural decisions."
   crates = []
   ```

4. **Create the directories** referenced by the config:
   `mkdir -p docs/adr/arc docs/adr/stale`.

5. **Write your first ADR** as `docs/adr/arc/ARC-0001-decision-title.md`.
   Run `adr-fmt` (no flags) to print the governance reference, which
   includes the ADR template and the rule catalogue.

6. **Validate.** `adr-fmt --lint` from anywhere inside the repository.
   Require the header to report zero warnings; exit `0` alone is advisory.

The tool walks up from the current directory to find `adr-fmt.toml`,
so step 6 works from any subdirectory.

## Usage

```bash
cargo run -- --lint
cargo run -- --tree
cargo run -- --refs AFM-0001
cargo run -- --context adr-fmt
```

## Build

```bash
cargo build
cargo test
```

### Cargo.lock policy

`Cargo.lock` is committed to this repository deliberately. `adr-fmt`
ships as a binary, and a committed lockfile is the standard Rust
convention for binary crates: it pins exact transitive-dependency
versions so two clones of this repository produce byte-identical
binaries given the same toolchain. Library crates typically *do not*
commit lockfiles; binary crates do.

If you depend on `adr-fmt` as a binary (via `cargo install` or a
release artefact), `Cargo.lock` is what makes that build
reproducible. If you ever consume it as a library, your own
project's `Cargo.lock` takes over and this one is ignored.

## Governance

This tool's own design decisions live in `docs/adr/adr-fmt/` (prefix
`AFM`). AFM-0038 records the 0.2.0 warning-output break under AFM-0036.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option (SPDX: `Apache-2.0 OR MIT`). Contributions are
accepted under the same dual licence by default.
