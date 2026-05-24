# INSCRIPTION — Arc 233 — Substrate diagnostic-richness: errors as teaching values

**Status:** SHIPPED 2026-05-23 night late. Closes arc 233. 14 sub-stones shipped in one session post-compaction.

**Driver direction at open** (2026-05-23 evening):

> *"we believed we had remarkable errors - we don't - we need to raise the bar"*

The wall. The substrate's error-emission was opaque text where it claimed to be diagnostic. Arc 232's defprotocol work surfaced — over ~30-50 min of investigation cost in one session — that wat's errors LOSE INFORMATION at exactly the moments richer diagnostics would teach fastest:

- `NotCallable { got: "wat::core::keyword" }` — named the TYPE, lost the VALUE
- Type-mismatch errors carried `&'static str` placeholders, not the actual value
- The bracket-syntax trap door (`[-> :T]` invented vs canonical `-> :T`) had no error-driven catch
- The FM 2-bis probe discipline was partly a WORKAROUND for the diagnostic gap

The tax compounded across substrate sessions. Per `feedback_failure_engineering` + `feedback_any_defect_catastrophic`: a structural problem costing 30-50 min per session × N sessions = real liability, not polish. We raised the bar.

## What "remarkable errors" means (the target — all four pieces DELIVERED)

| # | Piece | How it landed |
|---|---|---|
| 1 | Errors are structured values, not formatted strings | Stone 233.1 minted `ValueSnapshot { type_name, rendered, provenance }`; swept 282+ RuntimeError construction sites |
| 2 | ValueSnapshots replace `&'static str` | Stone 233.1 + 233.2.j ValueSnapshot::of_tracked addition for TrackedValue-aware error construction |
| 3 | Provenance tracking on Values | Stones 233.2.a/b/c/d/f/g/h/i/j/k/l/e — Provenance enum with 4 variants now populated; SymbolBound + Literal flow via AST-derived mechanism on the sealed substrate |
| 4 | Errors-as-EDN wire protocol | Stone 233.3 — runtime_error_to_edn for all 28 RuntimeError variants; HARD CUT at fork.rs + spawn_process.rs (replaces Display text with `#wat.kernel/<Variant>` envelopes on stderr) |

Arc 233's complete thesis reached **empirical delivery**. Errors carry source-coordinates (line + col + file), let-binding lineage (binding_span + head_span), producer-attached provenance (RuntimeBuilt for the 5 producers tagged), and serialize as machine-consumable EDN across IPC boundaries.

## The 14 sub-stones (commit chain)

| Stone | Commit | What landed |
|---|---|---|
| 233.1 | `13b9166` | ValueSnapshot mint + 282-site RuntimeError sweep (16/16) |
| 233.2.a | `7cfeff1` | Provenance enum + Value::Tracked variant + transparency contracts (Shape C; 16/16) |
| 233.2.b | `9cc278c` | keyword/from-string producer tag (12/12; minimum-viable proof) |
| 233.2.c | `c0f41f6` | 4-producer sweep (from-holon, edn::read, recv, try-recv; 14/15) |
| 233.2.d | `c4dc8f4` | substrate-symmetry uniform list_span (167 sigs; 12/13) |
| 233.2.f | `51d83e1` | apply Tracked-unwrap defect fix (8/8) |
| 233.2.g | `b2a3188` | Shape A pivot sub-DESIGN — TrackedValue struct over Value::Tracked variant |
| 233.2.h | `38acd60` | TrackedValue struct mint + adapter (9/9) |
| 233.2.i | `8164629` | eval signature flip — Result<TrackedValue, _> (107 files; 10/10) |
| 233.2.j | `c16419e` | eval_inner cascade — 383 sites + 5 producers + Phase 5 bind_let_binding (11/11) |
| 233.2.k | `be7ceaa` | **Value::Tracked variant DELETED**; Environment stores TrackedValue (12/12; arc216 stone1 7 probes AUTO-RESOLVED) |
| 233.2.l | `429c648` | **`#[wat_value]` proc-macro SEAL** — meta-class closure (12/12; trybuild 5 fixtures) |
| 233.2.e | `5d3d43f` | AST-derived provenance — Literal{span} + SymbolBound{binding_span, head_span} (12/12; correction round delivered 12/12 vs initial 11/12) |
| 233.3 | `48afb31` | Errors-as-EDN — 28 RuntimeError variant arms + provenance/snapshot helpers + wire integration (11/11) |

**Calibration:**
- 14 sub-stones shipped in one session (post-compaction continuation)
- Sonnet calibration trend was below predicted bands throughout (h: 3:12 vs 15-30; i: 64 vs 90-150; j: ~180 vs 90-150 with Phase 5; k: ~22 vs 60-120; l: ~63 vs 45-90; e: ~89 + ~2 correction vs 90-150; 233.3: ~35 vs 60-120)
- ~31 commits today (sub-DESIGN + probe + BRIEF/EXPECTATIONS + SHIPMENT + SCORE per stone, plus paperwork)

## Disciplines surfaced + inscribed during arc 233

### Failure-engineering ✅✅✅ standard (elevated from convention to structural)

User invoked `scratch/FAILURE-ENGINEERING.md` doctrine post-Stone-233.2.i. The four-questions verdict against the standard disqualified probe-only enforcement (catches AFTER construction; fails Honest) and qualified the proc-macro structural seal (compile error AT construction; passes all four).

**The annihilation table** (now empirically validated):

| Standard | Mechanism | Status this arc |
|---|---|---|
| ✅ | Convention — "remember to call `.inner()`" | failed 3+ times this session (Stone 233.2.f apply fix; "intentional gap" framing; arc-234 scope inflation) |
| ✅✅ | Convention + CI — lint catches post-hoc | partial — probes detected regressions mid-flight |
| ✅✅✅ | Structural — compile-error AT construction OR variant absent | Stone 233.2.k (instance closure) + Stone 233.2.l (meta-class closure) DONE |

After 233.2.l: the SITUATION that produces the trap-door (Value variant wrapping another Value with metadata) cannot be constructed in source AND cannot be RE-INTRODUCED at future authoring time without explicit per-variant `#[wat_value(allow_wrapping = "reason")]` ceremony with mandatory non-empty reason string.

### Partial-state-grading discipline (`feedback_partial_state_grading.md`)

Minted post-Stone-233.2.i ("don't lose good work because it took longer to deliver than we expected"). VINDICATED TWICE this arc:

- **Stone 233.2.j Phase 5** (unplanned) — sonnet honestly surfaced the let-binding provenance regression mid-flight; shipped two complementary fixes (bind_let_binding re-wrap + Value::into_tracked extraction) + probe-3-exempt mechanism with documented expiration at 233.2.k.
- **Stone 233.2.k unplanned additions** — eval_let return type flip + apply_tracked_callee helper (probe 3 caught the 7th provenance-stripping boundary BRIEF didn't enumerate; sonnet shipped inline).

On STOP-3 / time-box / "longer than expected": GRADE never auto-revert. SendMessage sonnet first; preserve honest work; commit green tree if possible.

### Three resequencings (FM 11 deferral catches)

| Original framing | Caught how | Reframed to |
|---|---|---|
| "arc 234 candidate" (substrate-symmetry as separate arc) | User challenge: "is 234 warranted or just a member of 233?" | Stone 233.2.d sub-DESIGN |
| Shape C only (Value::Tracked variant — original DESIGN) | Stone 233.2.f trap-door + class-incidence audit | Shape A pivot sub-DESIGN at 233.2.g |
| Probe-only enforcement of pattern-match discipline | FAILURE-ENGINEERING.md ✅✅✅ standard verdict | Stone 233.2.l proc-macro structural seal |

Each reframe was the four-questions discipline catching scope inflation OR convention-only enforcement that the doctrine rejected.

### Agency-attribution catches (4 recurrences)

| # | Date | Surfaced via | Dimension |
|---|---|---|---|
| 1 | 2026-05-13 | shadow-channel framing | VERBAL (LLM quoted user's frame as own) |
| 2 | 2026-05-17 | spawn-program framing | VERBAL |
| 3 | 2026-05-19 | surface-area-identical framing | VERBAL |
| 4 | 2026-05-23 | Song #25 Bad Guy "we chose the hard path" | **AGENCY** — user invoked discipline; discipline produced verdict; LLM narrated verdict as own choice; Oracle/vase frame |

Fourth dimension named: when a discipline (four-questions, doctrine cascade, etc.) produces a verdict, NAME THE DISCIPLINE'S WORK explicitly — "the discipline produced V" / "only Path X passed; we executed V" — NOT "we chose V." The orchestrator owns EXECUTION; the discipline owns the VERDICT.

Song #26 (Elevator Operator) corrected the frame: "lever is held not owned." Song #27 (We Got The Moves) honored what the lever produces (competence in execution patterns). Song #28 (Whatever It Takes) honored the COST of keeping the ratchet turning.

### Substrate-as-teacher cascade pattern (per FM 15)

Cargo enumerates the worklist. Each substrate-wide change ships as: short BRIEF + run cargo test --no-fail-fast → read errors → apply rule → iterate → green. The 383-site cascade in Stone 233.2.j is the load-bearing example.

Per `docs/SUBSTRATE-AS-TEACHER.md`: failures are diagnostic; cargo fail-count IS the progress meter. The 233.2.j cascade went from 0 errors → cascade-introduced ripples → 0 errors via iteration; the substrate's compile errors enumerated the next batch each round.

## What this unblocks

- **arc 232 defprotocol** — was paused at Stone 232.0a waiting on arc 233's substrate. Now UNBLOCKED. Resumes on the diagnostic-rich + sealed + EDN-emitting substrate. defprotocol's own dev cycle becomes consumer-side validation of arc 233's substrate work.
- **arc 217 Clojure-IPC bridge** — Clojure consumer parses `#wat.kernel/<Variant>` envelopes as `ex-info`-equivalent structured errors. Stone 233.3's wire format is the substrate this bridge needs.
- **wat-MCP horizon** — MCP tools consume structured errors instead of regex-matching opaque text. Provenance flows as machine-readable map keys.
- **MTG horizon, Truth Engine, trading-lab v2** — downstream domains get full provenance + structured errors for free.
- **arc 216 stone1 7 probes** (task #496) — AUTO-RESOLVED at Stone 233.2.k. Same trap-door class as 233.2.f apply fix; both gone with variant retirement.

## Honest deltas (affirmative scope-bounding)

- **recv/try-recv carrier-level provenance** — permanently lost at Stone 233.2.j Phase 6 (the wrap was nested in `Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged))))))`). Indirect coverage via SymbolBound when let-bound (common case). Original send-site span unrecoverable — lives in another execution context. Out of arc 233's scope; arc 217 Clojure-IPC bridge may revisit via different mechanism if cross-language origin tracking becomes load-bearing.

- **Chained provenance** (RuntimeBuilt → SymbolBound when let-bound producer result) — Provenance enum is flat per sub-DESIGN 233.2.e Decision 2; SymbolBound REPLACES stored RuntimeBuilt at lookup. Producer-context preserved in commits + SCORE + git-history; let-binding is the lexical scope context for diagnostics. Out of arc 233's scope; not tracked elsewhere because the substrate provides sufficient diagnostic context today via the three populated Provenance variants (Literal source-coords + SymbolBound lookup-lineage + RuntimeBuilt producer-attached).

- **eval_let_tail flip** — closed at Stone 233.2.e (the 233.2.k honest delta this stone resolved). Restored provenance in the tail-call path.

- **Destructure source per-element provenance** — slot gets binding_span pointing at LHS pattern; tracing slot back to source tuple's element-span is out of arc 233's scope. Slot-level binding provenance is sufficient for the diagnostic-richness target; deeper origin tracing not load-bearing today.

- **List call-form provenance** — dispatch fn determines result provenance (RuntimeBuilt for the 5 producers; otherwise Unknown). Not a "literal" per sub-DESIGN 233.2.e Decision 4. Out of arc 233's scope.

- **Type alias bypass of `#[wat_value]` syntactic scan** — Decision 1 of sub-DESIGN 233.2.l documented this as a known limitation. Opt-in escape hatch with mandatory reason string covers the legitimate-exception corner. Out of arc 233's scope.

- **`#[wat_value]` proc-macro scope** — applies only to `pub enum Value` in src/runtime.rs per Decision 3 of sub-DESIGN 233.2.l. HolonAST, WatAST, RuntimeError, and other enums could grow wrapping variants tomorrow if we don't apply the seal there too. Seal-per-target as classes surface; no language-wide structural ban. Out of arc 233's scope.

- **`WAT_ERROR_FORMAT=text` fallback** — NOT shipped per Stone 233.3 sub-DESIGN Decision 7 HARD CUT. The EDN envelope IS the wire format post-arc-233. If a downstream consumer surfaces needing text format, a separate follow-up arc adds the flag; not load-bearing today.

- **Nested error types** (`crate::hash::HashError` in `EvalVerificationFailed`) — Stone 233.3 uses lazy fallback `:error "<Display string>"` for nested errors. Future arc can deepen if structured access becomes load-bearing.

- **ValueSnapshot::of(&Value) sweep to of_tracked** — incremental migration per Stone 233.2.k. Each RAISE site that has TrackedValue can migrate from of(&Value) (gets Unknown provenance) to of_tracked(&TrackedValue) (gets producer-attached provenance). Out of arc 233's scope; landed incrementally as 233.3's wire-emission boundary required.

- **Cross-thread channel error EDN-on-the-wire** — receiver gets RuntimeError in-process via typed-channel decode; no wire at receive time. Wire emission is on the EMITTING side only (process exit). Out of arc 233's scope; not load-bearing.

## Cross-references

### Sub-stone artifacts
- `DESIGN-STONE-233.2.md` — main sub-DESIGN umbrella
- `DESIGN-STONE-233.2.d.md` — substrate-symmetry sub-DESIGN
- `DESIGN-STONE-233.2.g.md` — Shape A pivot sub-DESIGN
- `DESIGN-STONE-233.2.j.md` — eval_inner cascade sub-DESIGN
- `DESIGN-STONE-233.2.k.md` — variant retirement sub-DESIGN
- `DESIGN-STONE-233.2.l.md` — proc-macro seal sub-DESIGN
- `DESIGN-STONE-233.2.e.md` — AST-derived provenance sub-DESIGN
- `DESIGN-STONE-233.3.md` — Errors-as-EDN sub-DESIGN
- `SCORE-STONE-233.1.md` through `SCORE-STONE-233.3.md` — per-stone shipment records

### Probes (permanent regression guards in `tests/`)
- `probe_diagnostic_value_snapshot_in_errors.rs` (Stone 233.1)
- `probe_value_tracked_transparency.rs` — RETIRED at Stone 233.2.k (file deleted; variant gone)
- `probe_diagnostic_dynamic_keyword_invocation.rs` (Stone 232.0; survives arc 233)
- `probe_substrate_symmetry_list_span_threading.rs` (Stone 233.2.d)
- `probe_tracked_value_mint_contract.rs` (Stone 233.2.h)
- `probe_eval_signature_returns_tracked_value.rs` (Stone 233.2.i)
- `probe_stone_233_2_j_producer_migration.rs` (Stone 233.2.j; probe-3-exempt mechanism removed at 233.2.k)
- `probe_stone_233_2_k_variant_retired.rs` (Stone 233.2.k)
- `probe_stone_233_2_l_wat_value_seal.rs` (Stone 233.2.l)
- `probe_stone_233_2_e_ast_derived_provenance.rs` (Stone 233.2.e)
- `probe_stone_233_3_runtime_error_edn.rs` (Stone 233.3)
- `crates/wat-macros/tests/ui/*.rs` — 5 trybuild fixtures for `#[wat_value]` seal (Stone 233.2.l)

### Substrate modules new this arc
- `src/runtime_error_edn.rs` (Stone 233.3) — 375 lines; 3 pub fns + emit helper
- `crates/wat-macros/src/wat_value.rs` (Stone 233.2.l) — 249 lines; structural-seal proc-macro
- Provenance enum + TrackedValue struct + BoundEntry struct + value_snapshot_to_edn + provenance_to_edn (helpers in src/runtime.rs and src/runtime_error_edn.rs)

### Doctrines refined
- `scratch/FAILURE-ENGINEERING.md` ✅✅✅ standard — elevated from convention to structural via Stone 233.2.l verdict
- `feedback_partial_state_grading.md` (memory) — minted this arc; vindicated twice
- `docs/SUBSTRATE-AS-TEACHER.md` (FM 15) — 383-site cascade in Stone 233.2.j as worked example
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF discipline applied throughout

### Songs inscribed
- `INTERSTITIAL-REALIZATIONS.md` § Song #25 Bad Guy + annotation (fourth attribution-blur catch)
- § Song #26 Elevator Operator (lever held not owned)
- § Song #27 We Got The Moves (collective celebration)
- § Song #28 Whatever It Takes (the price paid)

### Predecessor arcs
- arc 064 — assert-eq renders values + surfaces location (precedent for value-render in diagnostics)
- arc 113 — cascading runtime error messages (precedent for causal chain awareness)
- arc 116 — phenomenal cargo debugging (Failure → Diagnostic; precedent for printing infrastructure)
- arc 138 — errors carry point-in-code coordinates (precedent for spans on every error)
- arc 211b — panic-as-EDN (the AssertionPayload EDN serializer; Stone 233.3 generalized this)
- arc 217 — Clojure-IPC bridge (consumer of structured errors over EDN; unblocked by Stone 233.3)
- arc 092 — wat-edn v4 minting (the EDN serialization substrate)

### Relationship to arc 232 (the strategic pivot)

Arc 232 PAUSED at Stone 232.0a. The substrate work for `extract-classifier` + `Bind/inner` lift NOT shipped (only probe + DESIGN committed at `96bb6f4`). After arc 233 closes, arc 232 resumes:
- Stone 232.0a substrate ships against the diagnostic-rich + sealed + EDN-emitting substrate
- Stone 232.1 defprotocol BRIEF gets authored with full provenance support
- defprotocol's own dev cycle is the consumer-side validation of arc 233

## Closing voice — the wall we faced; the ladder we built

The user invoked the wall on 2026-05-23 evening: *"we believed we had remarkable errors - we don't - we need to raise the bar."* That was the moment arc 232 PAUSED and arc 233 OPENED.

Tonight, after 14 sub-stones shipped in one session post-compaction:
- Every Value flowing through eval_inner carries meaningful provenance — RuntimeBuilt (5 producers tagged), Literal (source coordinates), SymbolBound (binding lineage), or Unknown (escape contexts only).
- The trap-door class (Value variant wrapping another Value) is structurally annihilated at BOTH the current substrate (variant deleted) AND future-authoring-time (proc-macro seal) layers.
- Errors flowing across IPC boundaries are tagged EDN envelopes — `#wat.kernel/<Variant>` maps that downstream consumers (Clojure-IPC, wat-MCP horizon) parse as structured data.
- The discipline ladder rose: failure-engineering ✅✅✅ standard articulated + applied to a worked example; partial-state-grading minted + vindicated twice; agency-attribution catch dimension named (fourth recurrence).
- Songs #25/26/27/28 mark the operational soundtrack across this arc's emotional arcs (identity-ownership → play-as-operation → collective-celebration → price-paid).

The wall is no longer there. In its place: the substrate that teaches via remarkable errors, ratcheted into structural permanence, recordable as EDN over any wire.

The king sits on the throne the substrate-work earned tonight.

*Arc 233: SHIPPED. INSCRIBED. The disk holds the red ink. The summertime memories will never fade away.*

*Dop-död-död-dop. You better wave bye bye.*
