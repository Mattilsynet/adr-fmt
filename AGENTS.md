# AGENTS.md — adr-fmt

Repo-specific operational notes. General agent/OODA doctrine, bash hygiene,
and the Rust no-`//`-comments house style live in the global
`~/.config/opencode/AGENTS.md` — not repeated here. opencode is the only
agent surface for this repo.

## Section 1: Canonical Fleet Doctrine

### OODA Loop Roles
- **Copernicus** (Observe): Raw evidence gathering from environment, code, and external specs. Pure sensor; produces no hypotheses.
- **Feynman** (Orient): Produces ranked hypotheses with falsifiers; stress-tests against concrete examples.
- **Moltke** (Decide): Standing mission commander. Emits executable mission contracts, sets intent, boundaries, and abort criteria.
- **Hopper** (Act): Executes missions using Kent Beck TDD (red-green-refactor) with verify-before-claim discipline.
- **Linus** (Review): Mandatory pre-merge Rust reviewer for idiom conformance, type safety, unsafe soundness, and supply chain.
- **Hamilton** (Assurance): Architectural alignment and assurance reviewer running during CI wait windows.
- **Gardener** (GC): Post-mission cleanup specialist; reclaims transient scaffolding and closes completed mission beads.

### Priority Hierarchy
Tradeoffs strictly resolve in this five-tier priority order:
1. **Maintainability**: Pure trunk development, small deployable increments, minimal cognitive overhead, low complexity.
2. **Correctness by design**: Make illegal states unrepresentable via types, explicit state machines, and private invariant constructors.
3. **Response times**: Latency-sensitive read paths and prompt fact propagation across boundaries.
4. **Energy efficiency in code**: Minimize redundant polling, hot loops, unnecessary serialization, and idle CPU/memory burn.
5. **Features**: New functionality ranks last and must never compromise the higher tiers.

### Non-Interactive Shell Commands & Bash Hygiene
Subagents execute non-interactively. Commands that prompt for user confirmation stall execution indefinitely.
- Always use non-interactive and force flags: `cp -f`, `rm -f`, `rm -rf`.
- Streaming and batch mode: use `--batch`, `-y`, or `--quiet` where available.
- Stream separation: machine-readable findings route to `stdout`; diagnostics and logs route to `stderr`.

### Zero Plain Comments
In Rust source (`*.rs`), plain comments (`//` or `/* */`) are forbidden.
- Rationale belongs in commit messages, ADRs, or bead descriptions.
- Use `///` or `//!` contract doc-comments only when defining public API documentation (with required `# Errors`, `# Panics`, `# Safety` sections).
- Suppress lints with `#[expect(lint, reason = "...")]` rather than plain comments.

### Doctrine: "Make tools fast to iterate fast"
Developer and verification tooling must be compiled, ultra-fast Rust binaries operating directly on ASTs and files rather than slow interpreted wrappers or token-heavy in-context simulation. Fast tools enable high-frequency local feedback loops (INNER cadence) without friction.

### Doctrine: "Zero compliance theatre"
High-assurance testing techniques—such as property-based testing (proptest), fuzzing (cargo-fuzz), formal model checking, or fault injection—must be applied purposefully at critical serialization, concurrency, and storage boundaries (high-risk seams), not sprayed ubiquitously as box-ticking ceremony. Where type invariants and deterministic unit tests suffice, do not add compliance overhead.

## Section 2: Target-Specific Profile

### Target Classification & Entrypoint
- Target class: `attended-app` (as mapped in `sf-sdlc.toml`).
- Real entrypoint: `src/main.rs`.
- Canonical verification entrypoint: `scripts/verify.sh`
- A single-crate Rust binary (+ library `adr_fmt`): a read-only ADR
  template and link-integrity validator. It discovers `adr-fmt.toml` by
  walking up from cwd, then lints/inspects an ADR corpus. See `README.md`
  for modes, bootstrap, and usage.
- Exit codes adhere strictly to the fleet tri-state taxonomy:
  - `0`: clean pass / compliant / all assertions verified.
  - `1`: domain defect / violation / finding flagged.
  - `2`: unknown / environmental error / missing permission / indeterminate.
- Stream separation: structured machine-readable findings (TSV, JSON Lines)
  stream to `stdout`; operational telemetry, diagnostic logs, and error traces
  route to `stderr`.

### Verification Cadences (Three-Tier Cadence)
Verification is strictly tier-scoped. A claim is backed by the tier whose scope
matches the claim: sub-missions are backed by MID; epics and releases are backed
by BOUNDARY.

- **INNER** (every hopper TDD increment and per-review-round re-verification;
  changed crate ONLY; exit-code criterion: test + clippy exit 0):
  ```sh
  CARGO_TERM_PROGRESS_WHEN=never cargo test -p adr-fmt --locked --message-format=short
  CARGO_TERM_PROGRESS_WHEN=never cargo clippy -p adr-fmt --all-targets --locked --message-format=short -- -D warnings
  ```
  `--all-targets` is mandatory on clippy to catch test/bench/example lints.
  `--workspace` and `--all-features` are forbidden at this tier.

- **MID** (once at sub-mission completion before done-claim; changed crates
  plus their reverse-dependent closure; exit-code criterion: test + clippy exit 0):
  ```sh
  cargo test --all-targets --locked
  cargo clippy --all-targets --locked -- -D warnings
  cargo fmt --all -- --check
  ```
  `--workspace` is forbidden at this tier; verify stays scoped to the affected closure.

- **BOUNDARY** (once per epic before epic done-claim; full workspace; exit 0 across all):
  ```sh
  cargo build --all-targets --locked
  timeout 900 cargo test --locked --no-fail-fast
  cargo clippy --all-targets --locked -- -D warnings
  cargo fmt --all -- --check
  sh scripts/verify.sh
  ```
  - `timeout 900` is mandatory on the test line. Exit 124 is `Outcome::Surprise`,
    NEVER a test failure. Investigate the stall; do not fold it into a failure count.
  - `--no-fail-fast` is mandatory on the test line to ensure full blast-radius
    visibility in a single pass.

- `clippy::pedantic` is the standing bar (`[lints.clippy] pedantic = warn`
  in `Cargo.toml`), not an elevation — new code passes it with zero
  warnings.
- `rustfmt` runs on **stable defaults only** — do not add a
  `rustfmt.toml` or `clippy.toml`.
- `rust-toolchain.toml` pins channel 1.98 (clippy + rustfmt). Use it,
  don't bump without cause.

### Supply Chain Gates
`cargo deny check` and `cargo audit` are supply-chain gates; run
before publishing or bumping dependencies.

### Rustdoc Budget Gate
Run the same native check from the repository root locally and in CI:
```sh
comment-free --check-doc-budget --doc-advisory-words 80 --doc-max-words 120 --max-warning-files 0 .
```

Requires comment-free 0.2.0 at the canonical revision below:
```sh
cargo +1.98.0 install --git https://github.com/acje/comment-free --rev e45de7ef3b0fcd9a1ec299b9026b14fb5b0cf534 --locked comment-free
```

The read-only native gate recursively scans Rust sources under `.` with the
tool's build/hidden pruning: 80 prose words is advisory; 120 is enforced.
Fenced code is excluded by the tool. Summary-only output retains full totals
while suppressing finding details; diagnostics remain visible.
Native gate exits are 0 for pass, 1 for enforced breach, and 2 for
unknown/error, including undecided payloads or empty scope. Policy and its
implementation/tests/proofs belong upstream; repository checks establish
integration only. No rewrite mode runs.
Macro-generated docs without spelled `doc` tokens remain outside detection;
this is not proof of semantic documentation coverage or process-memory bounds.

### TigerStyle Construction-Path Inventory
Invariant-bearing domain types must enforce "illegal states unrepresentable"
by design. For each changed constrained type, review all construction routes:
1. Public fields / struct literals (reject if fields allow inconsistent mutation).
2. Constructors & builders (`new()`, `builder()`).
3. `Default::default()` (must yield a valid domain state or be omitted).
4. Conversions (`From`, `TryFrom`).
5. Serde deserialization (custom validation if raw wire data could bypass invariants).
6. Mutation routes (setters, `DerefMut`).

Independent booleans remain valid booleans; genuine optionality remains `Option`.
Do not invent artificial domain restrictions where none exist.

### Closed Error Enum Policy (C4.5/C4.6)
Public error enums MUST NOT carry `#[non_exhaustive]`. Variant sets are complete
within a major semver line, making unhandled error states unrepresentable at
compile time. Enforced mechanically via `non-exhaustive-check` from the canonical
`tripwires` repository.

### Delivery
`main` is protected (PR required). Changes land via a feature branch and
a pull request — never a direct push to `main`.

### Governance
This repo carries its own ADR corpus at `docs/adr/adr-fmt/` (prefix
`AFM`, 29 Accepted), with 10 retired stubs in `docs/adr/stale/`
(39 ADRs total). Start with [the corpus guide](docs/adr/README.md).
These ADRs are binding — read the relevant ones
before changing behaviour they govern. Read them with `adr-fmt --tree`
/ `adr-fmt --context adr-fmt` or directly as markdown; the config file
`adr-fmt.toml` lives at the **project root**, not inside
`docs/adr/adr-fmt/` — look there first, not next to the ADR files.

`cargo test`/`clippy` above verify code, not corpus health. Corpus
health has its own locally-runnable gate — run it before handing off
work that touches `docs/adr/`:

```sh
scripts/adr-lint-gate.sh
```

It discharges AFM-0003:R3: `adr-fmt --lint` is advisory and exits 0 on
findings by design, so the gate parses the `## Diagnostics: N
warning(s)` header and exits 1 when N exceeds the threshold (exit 2
when it cannot obtain a count or duplicate IDs prevent complete validation —
never conflated with clean). The
threshold defaults to **8** and is overridable
via `ADR_LINT_MAX_WARNINGS`. CI runs the same script.

Both the threshold and the parsed warning count must be a bare decimal
integer of **1 to 9 digits** (0–999999999); that is the widest range
the shell can compare without overflowing its fixed-width arithmetic.
Anything else — empty, non-numeric, negative, leading `+`, whitespace,
or more than 9 digits — is **no verdict: exit 2**, never a pass. An
explicitly empty `ADR_LINT_MAX_WARNINGS=''` is a malformed value, not
"unset"; only a genuinely unset variable falls back to the default 8.

### ADR Writing Style
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
in `## Context` or `## Decision`.
