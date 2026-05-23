# BRIEF — Arc 227 Stone 227.2 v3 — defrecord with N≥0 fields shipping canonical instance shape

**v3 supersedes v2.** v2 (commit `b4509cb`) shipped with `STOP-5b deferred` framing for N≥2 multi-field; SCORE claimed 14/14 PASS but tests only exercised N≤1; the macro PANICS at expand-time for N≥2. v3 ships the composition empirically proven by `tests/probe_diagnostic_macro_splice_from_let.rs` (commit `c18fa6b`) + `tests/probe_diagnostic_bundle_result_compose.rs` (commit `72367f1`).

**Stone scope:** rewrite `:wat::holon::defrecord` macro to produce the canonical typed-entities instance shape for ALL N including N≥2. No `STOP-5b deferred` language anywhere. No flat-Bind workaround. No "future stone" deferrals.

**Type:** Sonnet Mode A.
**Time budget:** 120-240 min target; 360 min STOP.
**Depends on:** Stone 227.2 v2 SHIPPED at `b4509cb` (will be REWRITTEN by this stone); diagnostic probes SHIPPED at `c18fa6b` + `72367f1` (the design substrate).

## v3 supersedes v2 — the failure that triggered this stone

v2 shipped INCOMPLETE. The macro at `wat/holon/defrecord.wat` errors at expand time for N≥2 fields with diagnostic `"defrecord v2 STOP-5b: N>1 fields require substrate iteration at macro expand time; deferred to future stone."`

That diagnostic is FALSE. The substrate DOES support the iteration. Two probes prove it:

1. **`tests/probe_diagnostic_macro_splice_from_let.rs`** — proves `~@(:wat::core::let [...] (:wat::core::map xs fn))` splices Vec<WatAST> built via `Vector/map + runtime quasiquote` correctly. Probe 2 specifically demonstrates the EXACT pattern needed for N≥2 multi-field field-bind synthesis.

2. **`tests/probe_diagnostic_bundle_result_compose.rs`** — proves `(:wat::holon::Bind classifier-atom (:wat::core::Result/expect (Bundle items) "msg"))` produces the canonical `Bind(Atom, Bundle(...))` instance shape with inner children preserved.

Both probes pass. The composition that ships canonical defrecord exists. v3 implements it.

**Discipline this stone enforces** per `COMPACTION-AMNESIA-RECOVERY.md` FM 2-bis (inscribed at `47472de`):
- STOP triggers are REJECTION criteria — no "surface as finding" softening
- Test cases bind to specific EXPECTATIONS rows
- Sonnet mirrors the probe patterns verbatim — does not invent substitute compositions

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- DO NOT commit. DO NOT touch holon-rs. DO NOT touch wat-edn.
- **HARD CUT** — single-arg form stays retired; flat-Bind workaround DELETED.

## BASH DISCIPLINE

- ONE cargo command at a time, foreground; no piping
- 5 known signal-handler test hangs (task #413) — skip per Verification

## The design substrate — both probes

### Probe 1 reference (`tests/probe_diagnostic_macro_splice_from_let.rs`)

**MUST READ** before writing the macro. Probe 2 in that file shows the exact iteration shape:

```
~@(:wat::core::let
    [forms (:wat::core::map xs
             (:wat::core::fn [x <- :wat::core::i64] -> :wat::WatAST
               (:wat::core::quasiquote
                 (:wat::core::unquote (:wat::core::*'2 x 10)))))]
    forms)
```

Adapt for defrecord field-bind synthesis: each iteration produces a WatAST representing one field-Bind form; the splice fans them into the surrounding Bundle constructor.

### Probe 2 reference (`tests/probe_diagnostic_bundle_result_compose.rs`)

**MUST READ** before writing the macro. Probe 1 in that file shows the canonical instance composition:

```
(:wat::holon::Bind
  (:wat::holon::Atom (:wat::holon::to-holon "test::Foo"))
  (:wat::core::Result/expect -> :wat::holon::HolonAST
    (:wat::holon::Bundle [field-bind-a field-bind-b])
    "Bundle should not overflow"))
```

`:wat::holon::Bundle` returns `Result<HolonAST, CapacityExceeded>` BY DESIGN (arc 037 Kanerva-capacity enforcement; see `src/runtime.rs:15244-15268`). The macro uses `:wat::core::Result/expect` to acknowledge the discipline + unwrap. The Result-discipline is NOT a flaw; the flat-Bind workaround in v2 was sonnet ducking arc 037.

### The complete composition v3 must ship

```
(:wat::core::defmacro
  (:wat::holon::defrecord
    (fqdn   :AST<wat::core::nil>)
    (fields :AST<wat::core::nil>))
  `(:wat::core::do
     (:wat::core::defn ~fqdn [~@fields] -> :wat::holon::HolonAST
       (:wat::holon::Bind
         (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
         (:wat::core::Result/expect -> :wat::holon::HolonAST
           (:wat::holon::Bundle
             [~@(:wat::core::let
                  [pairs       (extract-field-pairs-from fields)
                   field-binds (:wat::core::map pairs
                                 (:wat::core::fn [pair] -> :wat::WatAST
                                   (:wat::core::quasiquote
                                     (:wat::holon::Bind
                                       (:wat::holon::Atom (:wat::holon::to-holon
                                                            (:wat::core::unquote (name-of pair))))
                                       (:wat::holon::Atom (:wat::holon::to-holon
                                                            (:wat::core::unquote (var-of pair))))))))]
                  field-binds)])
           ~(:wat::core::string::concat "defrecord " (:wat::core::keyword/to-string fqdn)
                                       " instance: Bundle capacity exceeded"))))
     (:wat::core::defn ~<predicate-fqdn-derived-via-existing-code>
                       [v <- :wat::holon::HolonAST] -> :wat::core::bool
       (:wat::holon::is? v ~(:wat::core::keyword/to-string fqdn)))))
```

The exact iteration over `fields` and the `name-of`/`var-of` decomposition is sonnet's specific work — but the SHAPE is locked by the probes. Sonnet must compose using `:wat::core::map` + runtime quasiquote + `~@(let ...)` splice + `Result/expect`. NO substitute composition.

## Your scope (sonnet)

### Phase 0 — Read the design substrate (MANDATORY)

Read in order:
1. `tests/probe_diagnostic_macro_splice_from_let.rs` (especially probe 2)
2. `tests/probe_diagnostic_bundle_result_compose.rs` (especially probe 1)
3. `wat/holon/defrecord.wat` (current v2 implementation; this is what gets rewritten)
4. `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis (the discipline this stone enforces)

The probes are NOT examples to skim — they are the WORKING COMPOSITION you mirror. Your defrecord macro body MUST use the splice pattern from probe 1 + the Result/expect pattern from probe 2 + the canonical instance shape Bind(Atom, Bundle(field-binds)) for ALL N including N≥2.

### Phase 1 — Rewrite `wat/holon/defrecord.wat`

Replace the v2 macro body entirely. The new body:
- Single 2-arg head: `(defrecord <fqdn> <fields>)`
- Constructor branch handles N=0, N=1, AND N≥2 UNIFORMLY via the splice composition
- N=0 case: empty field-binds Vec → empty inner Bundle (`Bundle []` returns `Result::Ok(Bundle())`)
- N≥1 case: Vector/map over field pairs → Vec<WatAST> of field-Binds → spliced via `~@(let ...)` into Bundle's args
- Bundle wrapped in `Result/expect` to acknowledge Kanerva-capacity discipline
- Predicate branch unchanged from v2

DELETE the v2 fallback code at lines 113-127 (the `(:wat::core::if (:wat::core::= n 3) ...)` branch + the `STOP-5b deferred` Option/expect bail). The composition handles N≥2 uniformly; no special-cases.

### Phase 2 — Update doc-comment header

Doc-comment at top of `wat/holon/defrecord.wat`:
- Replace "STOP-5b finding" section with the canonical-shape inscription
- All N cases produce `Bind(Atom("ns::Foo"), Bundle(Bind(Atom("field"), Atom(value))...))`
- Cite probes `c18fa6b` + `72367f1` as the design substrate
- Note Result/expect discipline per arc 037

### Phase 3 — Migrate + extend probe tests

Edit `tests/probe_arc227_stone2_defrecord.rs` (or rename if appropriate):
- ADD tests for N=2 (`[a <- :i64, b <- :String]`) and N=3 (`[a <- :i64, b <- :String, c <- :bool]`)
- For each new test, verify INSTANCE SHAPE via `from-holon` roundtrip OR direct extract_classifier + Bundle/first traversal:
  - Constructor takes N typed args
  - Instance has classifier atom matching FQDN
  - Inner Bundle has N field-Binds
  - Each field-Bind has correct field-name + field-value
- Keep existing N=0 + N=1 tests; verify they STILL pass with the new uniform composition
- Verify N=0 case ships canonical `Bind(Atom("name"), Bundle())` — NOT v2's `Bind(Atom("name"), Atom(nil))` (that was the flat-Bind workaround being retired)
- Verify N=1 case ships canonical `Bind(Atom("name"), Bundle(Bind(field-name, field-value)))` — NOT v2's `Bind(Atom("name"), Bind(name, value))` (also retired)

**Each EXPECTATIONS row in v3 binds to a specific test fn.** Sonnet's SCORE must cite the test name per row.

### Phase 4 — Verification

Run each ONE AT A TIME, foreground:

```
cargo build --release -p wat
cargo test --release --test probe_diagnostic_macro_splice_from_let
cargo test --release --test probe_diagnostic_bundle_result_compose
cargo test --release --test probe_arc227_stone2_defrecord
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc226_stone1_type_predicates
cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cargo test --release --test probe_arc216_stone2_vector_roundtrip
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cargo test --release --test probe_arc216_stone4_predicate_composition
cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc143_manipulation
cargo test --release --test mvp_end_to_end
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty.

### Phase 5 — APPEND addendum to SCORE-STONE-227.2.md

Per `feedback_inscription_immutable`: DO NOT rewrite SCORE-STONE-227.2.md body. APPEND a v3 supersession addendum citing the disconfirming probes + this stone's commit.

### Phase 6 — Write SCORE-STONE-227.2-v3.md

Mirror SCORE-STONE-227.1b.md shape. **Every scorecard row cites the specific test fn that proves it.** Calibration record. No "honest delta" framing for partial coverage — if a row didn't ship, MARK IT FAILED and surface the actual blocker.

## STOP triggers (REJECTION CRITERIA — not permission-to-defer)

- **STOP-1 (UNEXPECTED compile error):** STOP and report. Sonnet does NOT iterate around unexpected errors.
- **STOP-2 (test failure beyond migrated probes):** STOP + diagnose; broken-by-this-stone framing per Stone 221.3 Delta 1a.
- **STOP-3 (360 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched):** STOP and report.
- **STOP-5 (N≥2 case STILL panics):** if your composition still produces an expand-time error for `(defrecord :ns::Foo [a <- :i64, b <- :String])`, you are NOT done. The composition IS provable per probe 2 of `probe_diagnostic_macro_splice_from_let.rs`. Iterate the composition; do NOT ship partial.
- **STOP-6 (canonical instance shape not produced):** if N=0 instance is not `Bind(Atom("name"), Bundle())` OR N=1 instance is not `Bind(Atom("name"), Bundle(Bind(field-name, value)))`, you are NOT done. The shape is non-negotiable.
- **STOP-7 (bash discipline):** cargo hang from accidental pipes.
- **STOP-8 (substitute composition):** if you invent a workaround that doesn't use `:wat::core::map` + runtime quasiquote + `~@(let ...)` splice + `Result/expect`, STOP. The probes prove the canonical composition works. Use it.
- **STOP-9 (historical artifact rewritten):** BRIEF-STONE-227.2.md (v1 + v2) + EXPECTATIONS-STONE-227.2.md + STONE-227.2-NOTES.md + STONE-227.3-NOTES.md + arc 232 DESIGN.md — all stay intact per `feedback_inscription_immutable`. APPEND addendums; do NOT rewrite.

## Out-of-scope

- Methods bundled in defrecord (Pattern 3 from `STONE-227.2-NOTES.md` — methods stay separate defns)
- Inheritance (Stone 227.3 RETIRED; protocols absorb the use case)
- `:with-<field>` immutable setters (future)
- `:invariants` (future)
- defprotocol / extend-type (arc 232)
- from-holon support for typed-Tuple return from multi-field structs (future)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## What this stone explicitly REJECTS

- Any "STOP-5b deferred" language in SCORE deltas
- Any "future stone" framing for the load-bearing N≥2 case
- Any "substrate iteration doesn't exist" claim (it does; the probes prove it)
- Any flat-Bind workaround for N=0 or N=1 (canonical Bundle shape is mandatory)
- Any test that exercises only N=0 + N=1 (must include N≥2 cases)
- Any SCORE row marked PASS that doesn't bind to a specific test fn proving it

We expect perfection. We annihilate failure. Sonnet ships the canonical defrecord OR sonnet STOPs and surfaces the SPECIFIC blocker preventing it.
