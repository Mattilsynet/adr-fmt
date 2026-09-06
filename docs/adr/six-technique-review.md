# Six-technique semantic review checkpoint

Date: 2026-09-06. Mission: `adopt-six-03` / `adr-fmt-rjo8.3`;
foundational slice `adopt-six-03a` / `adr-fmt-rjo8.10`;
remaining AFM slice `adopt-six-03b` / `adr-fmt-rjo8.12`;
factory core slice `adopt-six-03c` / `adr-fmt-rjo8.14`;
factory optional slice `adopt-six-03d` / `adr-fmt-rjo8.16`.
This is a review inventory, not generated governance or package completion.
All 29 active AFM ADRs and all ten retired stubs were read in full before
the three-document repair. The 15 active AFM-0001..0024 decisions now also
have source-reviewed repairs approved in `adr-fmt-unrx`. The eleven remaining
AFM-0029..0039 rows have source-reviewed repairs approved in `adr-fmt-rjo8.13`.
The twelve factory FGND/FCOM/FSEC repairs are approved in `adr-fmt-4ssb`.
The twelve optional factory bodies have been individually read and repaired,
with independent approval in `adr-fmt-t8g9`. Final package acceptance remains
with the commander, separately from these slice approvals.

T1 = standalone obligation; T2 = justified first parent/exact citations;
T3 = crate retrieval; T4 = concise current rationale/explicit tradeoffs;
T5 = inspectable evidence outside rules; T6 = lifecycle/truthful totals.
“Retain” is a read-level assessment, not independent approval. “Repair” names
this checkpoint's diff. Pending independent review is not an approval.

## Shared scope and lifecycle assessment

T3 for every active AFM row: `adr-fmt.toml:23–33` maps the AFM domain to
crate `adr-fmt`; no foundation or per-document `Crates:` narrowing is present.
Retrieve with `--context adr-fmt`, not `--context AFM`. Each active decision
governs this tool, so domain-wide scope is retained; no copied policy bundle
is introduced. Semantic completeness of external consumers is not established.

T6 for active rows: all remain Accepted; none is wholly replaced by the
repairs. All active slices keep rule identifiers and receive
`Last-reviewed: 2026-09-06`. No supersession edge or new status is invented.
AFM-0039 is a limited compatibility successor, not whole-ADR retirement.
Keyword grandfathering under AFM-0031:R5 is not permission to change strength.

## Active AFM matrix

Paths are `docs/adr/adr-fmt/<ID>-*.md`; each row covers the whole file read.
The T5 column distinguishes inspected local implementation from pending checks.
Source review is not semantic entailment proof; foreign citations remain
unverified unless explicitly stated. Execution evidence lives in mission beads.

| ADR | T1 obligation assessment | T2 parent / exact constraint | T4 rationale/tradeoffs | T5 evidence / remaining check | T6 specific disposition |
|---|---|---|---|---|---|
| AFM-0001 | Repair: default guidance spelling; explicit marker subject | Root justified by invariant/config/judgment split | Repair: concise rationale; explicit costs/risks | Read `src/lib.rs:397–427,484–574`; guidance 414–477; no efficacy claim | Retain root |
| AFM-0003 | Repair: lint scope, current stderr ownership/path | AFM-0001:R4 explicitly justified | Repair: remove fmt/clippy analogy; explicit costs/risks | Read `src/main.rs:18–23`, lib 342–373, gate 30–93; foreign evidence excluded | Retain usage/retrieval distinction |
| AFM-0004 | Repair: default-mode guidance spelling | AFM-0001:R1 explicitly justified | Repair: explicit costs/risks | Read parser 467–542, template 435–458/622–690, guidance 414–477; parity not semantics | Retain three-site structural obligation |
| AFM-0006 | Retain parser scope/reassessment | AFM-0004:R1 explicitly justified | Repair: explicit costs/risks; no universal regex claim | Read parser 427–542/1041–1143; no full Markdown/performance guarantee | Retain |
| AFM-0008 | Retain naming/foreign scope | AFM-0001:R2 explicitly justified | Repair: remove chronology/capacity assurance | Read naming 17–97, links 93–143, config 23–33; historical allocation excluded | Retain foreign support |
| AFM-0009 | Retain three verbs | AFM-0001:R1 explicitly justified | Repair: conditional tree membership; explicit costs/risks | Read links 146–212, nav 71–106; historical cherry-pit analysis unavailable | Retain |
| AFM-0011 | Retain first-yes classification | AFM-0001:R1 explicitly justified | Repair: remove primacy claim/history; retain reassessment duty | Read template 876–899; no model-behavior or foreign-corpus claim | AFM-0010 remains retired |
| AFM-0012 | Repair: current T019, unchanged grammar | AFM-0011:R3 explicitly justified | Repair: remove obsolete future/history claims | Read parser 1041–1143, template 795–899, context 337–354; numeric validity not semantics | Retain |
| AFM-0013 | Retain single CLI owner | AFM-0001:R1 explicitly justified | Repair: current five modes, costs/risks | Read lib 62–114/594–643; no depth/guidelines flags, no size measurement | AFM-0002 remains retired |
| AFM-0015 | Preserve all four prior rules unchanged | AFM-0008:R2 exact citation added | Repair: explicit costs/risks; preserve no-foundation rationale | Read context 115–169/337–354, config 23–33; external applicability excluded | Preserve prior all-layer repair |
| AFM-0016 | Repair: selected-marker root, library stderr/main exit | AFM-0003:R1 explicitly justified | Repair: current containment rationale; explicit limits | Read containment 111–198, config 252–268, lib 568–591; races/deadlines excluded | No new security policy |
| AFM-0017 | Repair: ParseError, entry P001, library merge | AFM-0003:R2 explicitly justified | Repair: no silent-entry-skip claim | Read parser 105–330, lib 282–350; no exhaustive race reproduction | Retain advisory namespace |
| AFM-0020 | Retain structural first-reference rules | AFM-0009:R2 explicitly justified | Repair: no retired exception; selection versus grouping | Read nav 71–173, context 172–244/337–364; entailment not established | Retain live/terminal distinction |
| AFM-0022 | Retain stub-only protocol | AFM-0003:R1 explicitly justified | Repair: compact current rationale, costs/risks | Read template 475–620, nav 149–160; retirement prose still human-reviewed | Retain |
| AFM-0024 | Repair: R3 states surface without retired authority | AFM-0012:R4 explicitly justified | Repair: no retired seam authority; explicit costs/risks | Read refs 32–101, lib 292–317; no semantic critique guarantee | Retain workflow |
| AFM-0026 | Repair: replace R8 history with typed probe obligation; move commit logs out | Explain AFM-0006:R1 parser parent; retain support despite advisories | Repair: current API rationale and explicit tradeoffs | `src/lib.rs:33–54`, `src/containment.rs:32–39`; smoke probes | No scheduled v0.2 accessor migration |
| AFM-0027 | Repair: public Diagnostic path, known-marker helpers, no embedded oracle log | Repair: AFM-0026:R1 first; Accepted API is actual input constraint | Repair: remove Proposed/skeleton/version speculation | `src/config.rs:157–177,252–268`; no downstream build | Narrow amendment; keep support refs |
| AFM-0028 | Repair: trait subjects explicit; no pending implementation claim | AFM-0026:R1 fixes governed public set | Repair: implemented traits, present rationale and costs | `src/config.rs:186–220`; no external shim verification | Correct immutable-Accepted claim |
| AFM-0029 | Retain atomic replacement / partial amendment | AFM-0022:R1 explicitly justifies retired representation | Repair: citation-renumbering cost; remove deferred-enforcement claim | Read parser 734–761/1337–1349: clause-level Supersedes rejected; absorption remains judgment | Retain narrow amendment path |
| AFM-0030 | Repair: each rule names pardosa GEN/PAR scope and consolidation condition | AFM-0022:R1 parent; identify existing scoped exception to AFM-0029:R3 | Repair: current conditional disposition, no unsupported metrics | Read local config 23–33 and parser 734–761; PGN corpus/inventory/acceptance evidence unavailable | Remains Accepted; no completion assertion or foreign policy rewrite |
| AFM-0031 | Retain keyword strength / grandfathering | AFM-0012:R1 explicitly justifies tagged subjects | Repair: remove template/future-lint speculation; explicit tradeoffs | Read generated strength guidance and parser 977–1025; foreign RFC/GND/COM entailment not freshly verified | Retain grandfathering without strength inflation |
| AFM-0032 | Repair: distinguish external, crate, defining-module and test boundaries | AFM-0026:R1 explicitly justifies public model scope | Repair: remove draft correction and unverifiable downstream claims | Read model 159–875/1523–1559/2083–2311, parser 514–542; inventory below | Narrow amendment, no runtime redesign |
| AFM-0033 | Repair: raw accessor guarantee starts after parser trimming/absence handling | AFM-0003:R2 explicitly justifies warning severity | Repair: bounded-year limitation; no universal transposition detection | Read model 1243–1301/2244–2311, parser 569–581, template 241–276 | Retain year limits and separate verdict |
| AFM-0034 | Retain malformed tag versus parsed sequence distinction | AFM-0017:R1 explicitly justifies namespace | Repair: conditional warning multiplicity and explicit costs | Read parser 977–1143, template 773–874; no universal three-warning claim | Retain compatibility representation |
| AFM-0035 | Retain value/emptiness/provenance constraint | AFM-0032:R5 explicitly justifies shape-to-meaning extension | Repair: no scheduled v0.2 migration or eliminated-drift claim | Read model 414–440/2244–2311 and parser 569–581; downstream excluded | Current 0.3.x contract, no new migration |
| AFM-0036 | Retain all five rules and four-surface contract | AFM-0026:R1 explicitly justifies library membership | Repair: current-state definition and explicit compatibility cost | Read Cargo.toml 1–6, toolchain 1–4, golden guard 100–135; publication excluded | Current series remains 0.3.x |
| AFM-0037 | Repair: R5 obligation separated from uninspected measurement | AFM-0035:R4 explicitly justifies guard proof | Repair: explicit cost and syntax/entailment limits | Read golden guard 87–119 and parity 20–52; citation suites executed at mid tier, no new guard proof claimed | Retain all eight obligations |
| AFM-0038 | Repair: ongoing default obligation; release provenance outside rule | AFM-0036:R4 explicitly justifies output transition | Repair: explicit detail cost and resource limitations | Read output 131–210 and lib 354–373; skipped checks not complete validation | 0.2.0 limited successor remains applicable in 0.3.x |
| AFM-0039 | Retain seven runtime/guidance obligations unchanged | AFM-0036:R4 explicitly justifies break record | Retain costs; label risks and add inspectable evidence | Read lib 282–381/484–574, context 115–244/337–369, template 826–874, guidance 133–178 | Accepted limited successor retained |

The AFM author/source-review matrix is complete for all 29 active decisions
and the ten retired stubs, within the scopes above. Independent approval is
recorded in `adr-fmt-unrx` for the foundational slice and `adr-fmt-lv1e` for
AFM-0026/27/28; AFM-0029..0039 approval is recorded in `adr-fmt-rjo8.13`. This does not establish
external citation entailment, PGN consolidation, downstream builds, publication,
universal construction safety or semantic efficacy. Generated tier-name/order
prose reconciliation and its verification are tracked in `adr-fmt-rjo8.18`.

### AFM-0032 construction-route inventory

Scope: read-level assessment of `AdrId` and `AdrRecord`, not a new type design
or executed compile-fail proof. Their fields are private to `model`, not to
every crate caller. Invalid source observations remain legitimate record data.

| Route | Evidence and boundary assessment |
|---|---|
| Public fields / literals | Model 159–162/335–385: private outside `model`; defining-module literals can bypass constructor validation |
| ID constructors / conversions | Model 221–284/1523–1559: try_new validates, TryFrom and parsing helpers delegate; no FromStr or unchecked production conversion found |
| Record assembly | Model 628–685, parser 514–542: crate-visible trusted assembly, not parser-only visibility or a cross-field validator |
| Clone | Both types derive Clone; duplicates supplied values, does not independently validate them |
| Default / serde / public builder | Absent on both types in inspected model implementation |
| Mutation / DerefMut | No production mutable accessor or DerefMut; defining-module field writes remain possible; test-only mutation at model 717–835 |
| Test construction | Model 703–715/837–875: unchecked ID and record sentinels deliberately admit an empty prefix |
| Valid and invalid cases | Model 2083–2144: valid CHE-0042, rejected lowercase/short/long prefix and 10000; model 2220–2240 admits independent title/line facts |

Assessment: accept the narrowed external-boundary contract, not the former
universal claim. Tests named above are inspectable boundary evidence, not a
claim that every possible route was executed. Broader internal invariant design
requires separate authority; this migration does not remove fixture routes.

## Retired stub review

All ten files in `docs/adr/stale/` contain preamble, Retirement and only allowed
optional Supersedes lineage. T1–T5 active-decision decoration is inapplicable:
these are historical dispositions, not rules to retrieve or parents to invent.

| Stub | Disposition read | Remaining semantic caveat |
|---|---|---|
| AFM-0002 | Superseded by AFM-0013 | Mode-count language is retirement history |
| AFM-0005 | Deprecated, no implementation | No active safe-write authority |
| AFM-0007 | Superseded by AFM-0014 | Repaired: current retired disposition, no mode/dead-code claim |
| AFM-0010 | Superseded by AFM-0011 | Historical tier rationale retained |
| AFM-0014 | Superseded by AFM-0021; Supersedes AFM-0007 | Repaired: no surviving stub rules; pointers to live authority |
| AFM-0018 | Deprecated, never left Draft | No new L020 commitment |
| AFM-0019 | Deprecated evidence schema | Repaired: source-read T022 residue detector, no evidence schema revival |
| AFM-0021 | Superseded by AFM-0024; Supersedes AFM-0014 | Reach current workflow through AFM-0024 |
| AFM-0023 | Deprecated trigger language | No semantic-lint revival |
| AFM-0025 | Deprecated moratorium | Prior Related-section removal preserved |

## Factory core individual assessments

All twelve core files were read in full before editing (original lines 1–26).
Each now contains its own root rationale and explicit benefits, costs, risks
and inspectable evidence beside, not inside, the rules. These are author
assessments approved in `adr-fmt-4ssb`, not semantic or security guarantees.

T3 for each row: `software-factory/adr-fmt.toml:7–29` makes FGND, FCOM and
FSEC foundations. All twelve apply to every mapped profile, including the
remapped adopter; optional domains remain outside core-only retrieval. Read
`verify.py:11–22,47–106,109–123` for exact membership and adopter checks, and
`test_verify.py:19–152` for negative cases. No configuration or verifier change
belongs to this slice. The expected seven profile counts remain 12/14/15/17/14/17/14.

T6 for each row: Accepted, root identity, date, review date (2026-09-06), tier,
R1–R3 numbering and leverage layers remain unchanged. No decision is replaced;
retirement/supersession is inapplicable, not omitted migration work. Independent
retirement remains possible without fabricating parent edges. Adopter acceptance
and ownership remain separate decisions (`software-factory/README.md:5–6,88–124`).

Local evidence read: factory README (whole), SOURCES (whole), configuration
(whole), verifier (whole), regression tests (whole). SOURCES maps conceptual
lineage and exclusions, not implementation proof; its foreign sources were not
freshly read or used as binding parent authority. The per-rule review surfaces
below are proposed adopter evidence, not claims those reviews were executed.

| ADR | T1 individual obligation assessment | T2 root / exact constraint assessment | T4 rationale and tradeoffs | T5 inspected evidence / limits | T6 disposition |
|---|---|---|---|---|---|
| FGND-0001 | Retain all rules: brief owner/scope, executor assumptions, objective priority survive extraction | Independent brief-authority boundary; no required parent | Repair: priority/falsifier preparation cost and uncertain-assumption risk | SOURCES:26 ground synthesis; R1–R3 work-brief review criterion, no outcome proof | Retain root; no replacement |
| FGND-0002 | Retain observable success, MAY adaptation with MUST reporting, reassessment trigger | Feedback/reassessment independently scoped from brief contents | Repair: collection/escalation cost; missed-deviation risk | SOURCES:26; R1–R3 outcome/deviation/reassessment records, no executor-effectiveness measurement | Retain root; no replacement |
| FGND-0003 | Retain scope/owner, explicit optional adoption, reviewable obligations and maintenance | Policy lifecycle is independently applicable authority | Repair: continuing maintenance cost; Accepted is not adopter ownership | README:88–124 and config:7–69; R1–R3 owner/disposition review, no retirement-truth proof | Retain root; no policy retired |
| FCOM-0001 | Retain SHOULD domain operations and MUST boundary/dependency review | Module boundaries do not depend on a specific state or ownership design | Repair: connected-module review cost; abstraction-indirection risk | SOURCES:27; R1–R3 interface/call-site review, no adopter replaceability experiment | Retain root; no replacement |
| FCOM-0002 | Retain SHOULD types and MUST uncertainty/error semantics; independent flags remain legitimate | Observation/state meaning independent of module topology | Repair: explicit unavailable-state handling cost and alternate-route risk | SOURCES:76–81 rejects masking; R1–R3 construction/mutation/failure review, no universal type-safety proof | Retain root; no replacement |
| FCOM-0003 | Retain MUST alternatives/compatibility and SHOULD separate changes/simple mechanisms | Change planning independent of selected module design | Repair: consumer coordination cost; rollback is not restoration proof | SOURCES:27; R1–R3 alternatives/consumer/migration records, no migration execution | Retain root; no replacement |
| FCOM-0004 | Retain MUST authority/transfer/synchronization and SHOULD partitioning | Mutation authority independent of actor/storage topology | Repair: stale-authority reasoning cost; local locks not distributed exclusion | SOURCES:80–81 narrows single-writer mandate; R1–R3 mutation/transfer review, no fencing proof | Retain root; no replacement |
| FCOM-0005 | Retain MUST authority/guard proof and SHOULD test-first strength | Verification authority independent of the policy checked | Repair: fixture/execution cost and syntax-versus-semantics limit | verify.py:55–123; test_verify.py:102–152 plants/restores isolated membership violations; no adopter-behavior proof | Retain root; no guard edited |
| FSEC-0001 | Retain input validation, specific authentication/authorization and least capability; no permissive downgrade | Trust transitions independent of identity provider/deployment | Repair: integration cost; valid syntax is not authorization | SOURCES:28,32; R1–R3 trust/denied-operation review, no security/deployment testing | Retain root; no new security policy |
| FSEC-0002 | Retain classification/retention, conditional authenticated encryption and recipient-scoped diagnostics | Exposure/retention independent of authentication design | Repair: diagnostic/credential cost; no deletion/peer-verification assurance | SOURCES:28; R1–R3 inventory/transport/diagnostic review, no secret scan or crypto assessment | Retain root; no replacement |
| FSEC-0003 | Repair R2 subject to resource accounting and R3 to resource-budget evidence; MUST, conditions and exclusions unchanged | Baseline accounting independent of optional flow/static profiles | Repair: aggregate accounting cost; capacities remain adopter decisions | SOURCES:68–74; R1–R3 resource contracts/stress evidence, no measured high-water mark or universal bound | Retain root; no new resource constants |
| FSEC-0004 | Repair R1 subject to dependency intake; all strengths/conditions unchanged | Dependency acceptance independent of application trust mechanisms | Repair: intake/maintenance cost; clean advisories not safety or legal clearance | SOURCES:40–45 distinguishes versions from approval; R1–R3 intake/build-review/check/exception records, no dependency acceptance | Retain root; no replacement |

## Factory optional individual assessments

All twelve optional bodies were read in full before editing (original lines
1–26). Each now has its own independent-root rationale, explicit benefits,
costs, risks and inspectable evidence outside the three tagged rules. The
evidence pointers identify review surfaces, not executed adopter compliance.

T3 for each row: `software-factory/adr-fmt.toml:31–69` keeps all five optional
domains non-foundational with no per-document narrowing. FRST applies to
factory-rust/service/static; FFLO to factory-flow/service; FSTO to factory-storage;
FSTA to factory-static; FAGT to factory-agents. The independent subjects remain
separate roots, while domain mappings intentionally select each complete profile.
No additional selector, parent edge or policy bundle is introduced.

T6 for each row: Accepted, root identity, dates (2026-09-06), tier B, R1–R3
and all layers are retained. Seven subject clarifications preserve normative
strength and conditions; no whole decision is replaced, so no retirement or
supersession is warranted. Local acceptance still does not adopt policy on
behalf of another repository's owner.

Inspected evidence: README:15–34,52–125; SOURCES:24–82; config:31–69;
verify.py:11–22,47–123; test_verify.py:19–152. Source mappings are local
provenance/exclusion records; foreign sources were not freshly read and are
not binding parents. All paths below are under `software-factory/`.

| ADR | T1 individual obligation assessment | T2 root / exact constraint assessment | T4 rationale and tradeoffs | T5 inspected evidence / limits | T6 disposition |
|---|---|---|---|---|---|
| FRST-0001 | Repair R3 subject to Rust delivery; preserve MUST checks and SHOULD defaults/pedantic | Build inputs independent of API design or CI provider | Repair upgrade-evidence cost; pin is not safety | SOURCES:29 excludes exact compiler/CI; R1–R3 toolchain, inventory and check records needed, no adopter build | Retain root |
| FRST-0002 | Repair R1/R3 Rust subjects; retain SHOULD types, MUST unsafe review and documentation restrictions | Construction/documentation independent of compiler choice | Repair alternate-route and semantic-doc-review costs | SOURCES:76–78 private rationale; R1–R3 route/safety/rustdoc review needed, no compile-fail or soundness proof | Retain root |
| FFLO-0001 | Retain admission reservations, bounded waiters, explicit exhaustion and retry budgets | Admission independent of queue/executor implementation | Repair reservation-composition cost and application-specific drop risk | SOURCES:68–74 accounting/admission split; R1–R3 overload/ownership/retry evidence needed, no measured memory bound | Retain root |
| FFLO-0002 | Repair R2 subject to queue measurements; preserve all fields and SHOULD tuning | Measurement authority independent of admission mechanism | Repair instrumentation cost and copied-target risk | SOURCES:29,68–74 excludes universal targets; R1–R3 dated telemetry/scheduling evidence needed, no benchmark | Retain root |
| FFLO-0003 | Retain bounded work, deliberate yield, supervised termination and partial-I/O obligations | Supervision independent of queue/telemetry; no finite service lifetime | Repair stalled/partial-effect test cost; cancellation not rollback | SOURCES:53 distinguishes NASA from local adaptation; R1–R3 progress/shutdown tests needed, no running service tested | Retain root |
| FSTO-0001 | Retain authoritative state, read consistency and explicit storage-selection scope | Contract vocabulary independent of recovery implementation | Repair failure-model cost and acknowledgment/cache confusion risk | SOURCES:30,34–36 excludes backend/topology mandates; R1–R3 crash/read contracts needed, no durable-write proof | Retain root |
| FSTO-0002 | Retain mutation-boundary fencing, distinct retry windows and recovery cases | Recovery mutation authority independent of durability model | Repair representative-fixture cost and expired-dedup/cutover risk | SOURCES:30,34–36 recovery lineage; R1–R3 fencing/replay/corruption tests needed, no failover experiment | Retain root |
| FSTA-0001 | Repair R2/R3 no-allocation phase/instrumentation subjects; preserve scoped claim | Measurement envelope independent of allocator/pool | Repair reservation/instrumentation cost; persistent output excluded | SOURCES:49–51,56 scoped allocation; R1–R3 instrumentation needed, no measured zero-allocation or process bound | Retain root |
| FSTA-0002 | Repair R2 subject to pool capacity accounting; preserve identity/reset/overflow obligations | Reuse authority independent of allocation phase/timing claim | Repair generation/reclamation cost and ABA/bump risk | SOURCES:52,55,57–58 reclamation limits; R1–R3 handle-route/reset tests needed, no safety proof | Retain root |
| FSTA-0003 | Retain operation-specific work, latency assumptions and no_std exclusions | Predictability evidence independent of allocator choice | Repair per-operation analysis cost and idle-scan risk | SOURCES:52,54,56,58 distinguishes complexity/progress; R1–R3 complexity/timing evidence needed, no deadline guarantee | Retain root |
| FAGT-0001 | Retain bounded mission, permission and durable-secret-safe evidence obligations | Delegation boundary independent of review workflow/tracker | Repair coordination cost and readable-pointer versus truth distinction | SOURCES:37 excludes mandatory roles/tools; R1–R3 mission/permission/body review needed, no bypass test or secret scan | Retain root |
| FAGT-0002 | Retain tiered coverage, risk-scaled independent review and owner acceptance | Delivery acceptance applies without delegation | Repair closure cost and increment-versus-boundary risk | README:52–87 and test_verify.py:102–152 mechanical plants; R1–R3 review/owner acceptance still required | Retain root |

## Completion boundary and remaining package work

Local author/source-review migration covers all 29 active AFM decisions,
ten retired stubs and 24 factory decisions. No per-document assessment remains
uncovered. Optional factory independent approval is recorded in `adr-fmt-t8g9`.
Slice approvals do not substitute for final package review. Final boundary
execution is delegated to `adr-fmt-rjo8.18`; verification acceptance and
decision-owner acceptance remain with `adr-fmt-rjo8` / `adr-fmt-rjo8.3`.

The generated tier-name/order prose correction, golden/parity evidence and
review are tracked in `adr-fmt-rjo8.18`. The factory
verifier and configuration are unchanged by this slice: 24 ADRs / eight domains,
seven exact profiles and remapped adopter remain the mechanical contract.
No external implementation, security, timing, allocation, publication or foreign
source-entailment assurance follows from this completed local author review.
