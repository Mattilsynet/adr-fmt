# Source mapping and limitations

Research snapshot: **2026-09-06**. This corpus is newly authored synthesis,
not copied ADR text, and its F-prefixed IDs do not preserve upstream identity.
Distribution uses the enclosing repository's MIT OR Apache-2.0 terms. No
third-party prose excerpts are included. Missing license evidence is not
permission to copy and is not a legal clearance for derivative material.

The collection evidence is `adr-fmt-gh6o`; architecture interpretation is
`adr-fmt-e4pz`; corrections and adoption stress analysis are `adr-fmt-zqw7`
in the adr-fmt Beads store. Source revisions below come from that inventory,
not a claim that every source checkout was independently reread by the author.
Direct author reads covered gh-report GND-0002, COM-0005/0018, SEC-0003,
FLO-0007, RST-0004 and PGN-0016. Reads of gauntlet, active pardosa and
comment-free were permission-denied; those rows rely on the supplied evidence.
External research was performed by the evidence collector, not repeated here.

## Repository families

Paths in the source column are relative to the linked repository at its
recorded revision. “Adapt” means conceptual synthesis with local commitments
removed; “exclude” means deliberately not an adopted obligation.

| Family and revision | Source sections / IDs | Disposition and destination | License evidence / exclusions |
|---|---|---|---|
| [gh-report](https://github.com/Mattilsynet/gh-report/tree/4279ab2514d3c5ff0717936e9f09b18fa21f220f) `4279ab2514d3c5ff0717936e9f09b18fa21f220f` | `docs/adr/ground` GND-0001–0009 | Adapt intent, uncertainty, feedback, lifecycle into FGND-0001–0003 | MIT OR Apache-2.0; exclude GND-0010/0011's project consistency mandates |
| Same | `docs/adr/common` COM-0001–0038 | Adapt boundaries/information hiding into FCOM-0001; state/errors into 0002; trade-offs/evolution into 0003; ownership into 0004; SSOT/tests into 0005 | Exclude local stores, CI paths and blanket single-writer mandate; failure is never automatically empty success |
| Same | `docs/adr/security` SEC-0001–0007, 0009/0010/0013 | Adapt trust into FSEC-0001, confidentiality into 0002, resource lifecycle into 0003, intake into 0004 | Exclude mandatory adjacent private comments and universal exhaustion guarantees; event-log and WebSocket specifics not baseline |
| Same | `docs/adr/rust` RST-0001–0005; `docs/adr/flow` FLO-0001–0015 | Adapt FRST-0001/0002 and FFLO-0001–0003 | Exclude exact toolchain, internal checker, CI names, universal utilization targets and 50% failure heuristic |
| Same | `docs/adr/pardosa` PGN-0001/0004/0007/0016/0017/0023/0024 | Adapt authority, durability, fencing and recovery into FSTO-0001/0002 | Exclude .pgno encoding, NATS versions/headers, backend prohibitions and deployment topology |
| [cherry-pit](https://github.com/acje/cherry-pit/tree/c9d56bc80c8db98b96be4df2398c517913349fb7) `c9d56bc80c8db98b96be4df2398c517913349fb7` | `docs/adr/{ground,common,rust,security,pardosa,genome,cherry}`; predecessor GND/COM/RST/SEC and PAR/GEN/CHE families | Historical conceptual lineage of FGND/FCOM/FSEC/FRST/FSTO, not duplicate authority | MIT, copyright 2026 cherry-pit contributors; exclude application domain traits and retired storage identities |
| [gauntlet](https://github.com/acje/gauntlet/tree/797f7fb03a25ab63a53f3500a2ff89b3dc77bffe) `797f7fb03a25ab63a53f3500a2ff89b3dc77bffe` | `docs/design/2-supervision.md`, `3-actor.md`, `5-messaging.md`, `8-security.md`; `proposals/research/deterministic-allocation-generations.md` | Adapt capability/ownership to FSEC-0001/FCOM-0004, supervision to FFLO-0003, allocation identity to FSTA-0002 | No LICENSE found; no copied specification, AEM/Wasm topology or implementation requirements |
| [gauntlet-build](https://github.com/acje/gauntlet-build/tree/b40c209ad801fc02a9453a1db048c889fc325c71) `b40c209ad801fc02a9453a1db048c889fc325c71` | AEM documentation publishing companion | Evaluate as publication lineage for gauntlet, not an independent normative source | No license established; exclude Hugo/site structure and duplicate mandates |
| [pardosa](https://github.com/acje/pardosa/tree/09bdc68e156bbe5cf9cbbb7ff85d62e7576b6e9f) `09bdc68e156bbe5cf9cbbb7ff85d62e7576b6e9f` | `docs/spec/pardosa-1.0.md`, `docs/origin/design-notes-2026-04.md` | Adapt durable authority/recovery concepts to FSTO, ownership to FCOM-0004 | MIT OR Apache-2.0; active repo has no MADR corpus; do not invent active PAR obligations |
| [rescue-pardosa](https://github.com/acje/rescue-pardosa/tree/34a2cc2811c087db56d8efac15ec2bea3c8f2b36) `34a2cc2811c087db56d8efac15ec2bea3c8f2b36` | `docs/adr/0002..0023` | Historical recovery/authority lineage, represented by FSTO and active PGN evidence | No LICENSE found; exclude superseded implementation commitments |
| [old-pardosa](https://github.com/acje/old-pardosa/tree/a22f765c155b1edb8c157239cf9f39211b9da512) `a22f765c155b1edb8c157239cf9f39211b9da512`; local `acje/pardosa-stale-adr` | `pardosa.md`; unversioned `pardosa/`, `pardosa-file/`, `pardosa-schema/` | Evaluate as historical storage lineage only; no extra baseline rules | No license established; stale directory is not a Git repo and has no revision/permalink; excluded as normative authority |
| [opencode configuration](https://github.com/acje/.config/tree/b8cbfb1a5719945b6a1935e6d9e3293362415dd2) `b8cbfb1a5719945b6a1935e6d9e3293362415dd2` | `AGENTS.md`: resource contracts, comments, verification, delegation, evidence lifecycle | Adapt FSEC-0003, FFLO, FRST-0002, FAGT-0001/0002 | No LICENSE found; no copied prompts, mandatory bd, agent names or automation topology |
| [comment-free](https://github.com/acje/comment-free/tree/df97f29bb8fa9121c55b1537da600aeb21fbeb34) `df97f29bb8fa9121c55b1537da600aeb21fbeb34` | `docs/record-format.md`; Rust comment-checking tool contract | Operational influence on FRST-0002 and FCOM-0005 | MIT OR Apache-2.0; not installed or required here; syntactic acceptance is not permission for unnecessary private doc comments |

## Primary external sources

All ten source families are mapped below. Unversioned pages were observed
on the snapshot date; their URLs are readable primary references, not immutable
content hashes. Library versions identify reviewed documentation, not approved
dependencies. Any actual adoption requires independent supply-chain intake.

| Source | Supported concept and destination | Limits retained in this corpus |
|---|---|---|
| [TigerStyle](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md) | Startup reservation, bounded work and overload handling: FSTA-0001/0003, FFLO-0001 | Database-specific; no copied batch size, assertion density, function length or zero-dependency policy. Dropping arbitrary service requests is not automatically safe. Live main is mutable. |
| [Hard Mode Rust](https://matklad.github.io/2022/10/06/hard-mode-rust.html) (2022-10-06) | Injected resources and phase-scoped scratch: FSTA-0001/0002 | Toy implementation and lifetime/unsafe difficulties are not a general no_std prescription or safety proof. |
| [Static Allocation for Compilers](https://matklad.github.io/2025/12/23/static-allocation-compilers.html) (2025-12-23) | Finite processing chunks versus growing persistent output: FSTA-0001 | Startup heap allocation is not compile-time static storage; output arenas can grow and disk can exhaust. Compiler usefulness is exploratory. |
| [Static Allocation, Constant Work](https://matklad.github.io/2026/09/02/static-allocation-constant-work.html) (2026-09-02) | Configured-capacity scans and recycled identity: FSTA-0002/0003 | Idle CPU cost and logical stale handles remain; source P100 language is not measured universal wall-clock evidence. |
| [NASA/JPL Power of Ten](https://spinroot.com/gerard/pdf/P10.pdf) (Holzmann, 2006, Rule 2 rationale p2) | Bounded terminating work and checked execution: FFLO-0003, FCOM-0005; allocation discipline: FSTA-0001 | Intentionally nonterminating schedulers are exempt from terminating-loop bounds; the source asks proof they cannot terminate. Graceful service shutdown is our adaptation, not NASA's rule. C pointer/function/assertion restrictions are not imported. |
| [Embedded Rust Book: collections](https://docs.rust-embedded.org/book/collections/index.html) and [static guarantees](https://docs.rust-embedded.org/book/static-guarantees/index.html) | Type-encoded guarantees and capacity-aware collections: FRST-0002, FSTA-0003 | Zero-cost compares abstraction overhead, not total resource consumption; no_std does not prove no allocation. Large inline values can overflow stacks. |
| [Philipp Oppermann: Heap Allocation](https://os.phil-opp.com/heap-allocation/) | Allocator ownership, OOM and synchronization: FSTA-0002, FSEC-0003 | Kernel teaching example, not a general allocator recommendation; fragmentation, interrupt locking and exhaustion need local analysis. |
| [heapless 0.9.3](https://docs.rs/heapless/0.9.3/heapless/) | Inline fixed capacity and explicit operation failure: FSTA-0001/0003 | Vec.push is an example of constant time; LinearMap lookup is linear. Errors are operation-specific, not universally CapacityError. Inline storage may itself reside on the heap. |
| [static-alloc 0.3.2](https://docs.rs/static-alloc/0.3.2/static_alloc/) | Cumulative bump capacity and exclusive reset: FSTA-0002 | Individual deallocation does not reclaim capacity; reset requires lifecycle proof. Not a drop-in general allocator recommendation. |
| [lockfree 0.5.1](https://docs.rs/lockfree/0.5.1/lockfree/) | Progress and deferred reclamation distinctions: FSTA-0002/0003 | Uses owned-alloc and heap nodes. Lock-free is not per-thread latency; even wait-free abstract step bounds require hardware and scheduling assumptions for deadlines. |

## Coverage and conflict review

FGND (3), FCOM (5) and FSEC (4) form the 12-decision core. FRST (2),
FFLO (3), FSTO (2), FSTA (3) and FAGT (2) are the 12 optional decisions.
This is family-level curation, not a claim to reproduce every upstream rule.
Broad source ranges above describe evaluated themes, not independent approval
of every source sentence. Independently retireable decisions remain separate.

Resource claims are split deliberately: FSEC-0003 owns scope and aggregate
accounting; FFLO owns admission, telemetry and shutdown; FSTA owns the stronger
opt-in allocation/work proof. No resource constants are supplied. Adopters
must choose capacity and overload semantics. Tests must include at/over-limit
input, concurrent producers, stalled consumers, retry, cancellation, shutdown
and arithmetic overflow where applicable. Record measured high-water marks
and excluded resources; never promote a finite run to a universal bound.

Upstream SEC-0003's adjacent-comment instruction is not adopted. Meaningful
types/constants encode units and boundaries; API-required rustdoc remains
permitted, while private rationale lives outside source. COM-0005's error
masking is narrowed: missing permission or failed observation remains unknown,
not empty success. COM-0018's mandatory topology becomes explicit ownership
with justified synchronization. No upstream license uncertainty is resolved
by this document; review of originality and adoption obligations remains necessary.
