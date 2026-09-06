# Software-factory corpus

An original, additive baseline of **24 decisions across eight domains**.
It is not a replacement for adr-fmt's AFM corpus or an import of upstream
policy identities. Accepted means usable in this distribution; adoption by
another repository still requires that repository's decision owner.

Mechanically compatible with adr-fmt 0.3.0; all 24 decisions have individual
six-technique author-review assessments and repairs. The twelve core
FGND/FCOM/FSEC decisions and twelve optional decisions have independent
approval, with the optional slice recorded in `adr-fmt-t8g9`. Final boundary
evidence is tracked in `adr-fmt-rjo8.18`; commander review and owner acceptance
remain separate. See the
[migration guide](../docs/adr-format-proposals.md) and
[exact review inventory](../docs/adr/six-technique-review.md). A passing profile
verifier is not a semantic review of these 24 decisions.

## Profiles

Profiles are example **crate selectors**, not CLI flags. Only FGND, FCOM and
FSEC are foundations. Optional domains remain `foundation = false`.

| Selector | Core (12 ADRs) plus | Total |
|---|---|---:|
| `factory-core` | nothing | 12 |
| `factory-rust` | FRST | 14 |
| `factory-flow` | FFLO | 15 |
| `factory-service` | FRST + FFLO | 17 |
| `factory-storage` | FSTO | 14 |
| `factory-static` | FRST + FSTA | 17 |
| `factory-agents` | FAGT | 14 |

An ordinary Rust service receives neither storage, static-allocation nor
agent-delivery rules. Selecting static allocation does not install a crate,
allocator or no_std template. Selecting storage does not choose PC/EC,
JetStream or an event-store architecture. Profiles compose by listing an
adopter crate in each desired optional domain's `crates` array.

## Inspect from this checkout

Build the current source with `cargo build --locked` at the repository root.
Run these commands with **software-factory as the working directory**:

```sh
../target/debug/adr-fmt --lint
../target/debug/adr-fmt --tree
../target/debug/adr-fmt --context factory-service
```

Expect zero warnings, 24 ADRs, eight factory domains and no AFM records or
discovery-skip messages. Lint warnings are advisory: exit zero alone is not
a clean-corpus proof. The root `scripts/adr-lint-gate.sh` changes cwd to the
outer repository and therefore does not validate this nested corpus.

From the outer repository root, `python3 software-factory/verify.py` checks
exact corpus identity and all seven profiles, including every emitted rule.
It also copies the distribution and license files into temporary adopter
storage under `.ooda/tmp`, remaps `factory-service` to `my-service`, verifies
that copied corpus and context, and removes the temporary copy. Python 3 and
an existing `.ooda/tmp` directory are required; the verifier uses the binary
built at `target/debug/adr-fmt`, not an ambient installed version.

The verifier uses unconditional failures, including under Python optimization.
It parses every nonblank output record and compares exact domain, ADR and
rule multiplicities, including rule numbers and leverage layers. Malformed,
unexpected, missing or duplicate records are failures, not ignored output.
Its expected identifiers/layers describe this distribution; intentional policy
changes require a reviewed expectation update, not looser parsing.

Reproduce regression and end-to-end checks from the outer repository root:

```sh
cargo build --locked
python3 software-factory/test_verify.py
python3 -O software-factory/test_verify.py
PYTHONOPTIMIZE=1 python3 software-factory/test_verify.py
python3 software-factory/verify.py
python3 -O software-factory/verify.py
PYTHONOPTIMIZE=1 python3 software-factory/verify.py
git diff --check
scripts/adr-lint-gate.sh
```

The regression suite plants static-foundation inheritance and an additional
lint-valid R4 in isolated distribution copies. Each plant must exit nonzero
and its restoration must exit zero under ordinary Python, `-O` and inherited
`PYTHONOPTIMIZE=1`. It also injects malformed, duplicate, missing and altered
records directly into the parser checks. `unittest` checks are not removable
Python `assert` statements. Corpus originals are never mutated by these tests;
temporary copies are removed on both success and failure.

## Adopt into a fresh repository

1. Copy this complete directory's contents into the adopter root, retaining
   `adr-fmt.toml`, `docs/adr`, this guide and `SOURCES.md` together. Preserve
   the distribution's MIT/Apache license files from the outer repository.
   Record the source Git revision and selected profiles in the adopter's
   governance record. This is vendored documentation, not a runtime dependency.
2. In `adr-fmt.toml`, replace the selected example crate selectors with the
   actual crate name in every desired domain. A core-only consumer needs a
   mapping in FGND even though that domain is a foundation. Keep optional
   domains non-foundational. Unselected domains may stay registered with
   empty crate arrays, or be removed together with their directories.
3. Run a provenance-verified adr-fmt binary from the adopter root. There is
   no `init`, `--config`, `--profile` or `--guidelines` flag. No arguments
   prints guidelines; `--context` takes a crate name, not a domain prefix.
4. Check `--lint`, `--tree` and `--context <actual-crate>` for positive and
   negative membership. All copied configured directories must exist, and
   the corpus must be beneath its marker; `../` corpus escapes are invalid.
5. Resolve workload budgets, overload behavior, threat model, dependency
   approval and local verification commands before claiming compliance.
   No example profile supplies universal resource constants.

For an existing corpus, merge domain declarations deliberately rather than
overwrite its marker. Check prefix collisions and existing policies first.
An optional domain made foundational applies to every mapped consumer and
cannot be excluded per crate. Nearest-fit discovery can fall back to an outer
marker when a copied corpus is missing, so assert identity as well as warnings.

## Review and updates

Each document is a separately scoped root decision to allow independent
retirement; the domain grouping is the applicability boundary, not an
invented dependency hierarchy. Review source coverage and semantic conflicts
using [SOURCES.md](SOURCES.md), not just validator output. Updates are explicit
vendor diffs; upstream changes do not automatically alter adopter contracts.
Ordinary applications may allocate within declared budgets. Strict no-allocation
requires FSTA's scoped instrumentation; bounded work is not a latency proof.
