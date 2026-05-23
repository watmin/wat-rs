# EXPECTATIONS — Arc 227 Stone 227.2 — Multi-field defrecord + auto-accessors

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | defrecord macro accepts 1-arg OR 2-arg form | Variadic dispatch by arity; 1-arg = existing single-data; 2-arg = new multi-field |
| 2 | Single-data form (Stone 227.1b) STILL works | All 18 existing `probe_arc227_stone1_defrecord` tests pass unchanged |
| 3 | Multi-field constructor signature derived from field-list | `(defrecord :ns::T [a <- :i64, b <- :String])` mints `:ns::T` taking 2 typed args returning `:wat::holon::HolonAST` |
| 4 | Multi-field instance shape correct | Constructor produces `Bind(Atom("ns::T"), Bundle(Bind(Atom("a"), Atom(<a-val>)), Bind(Atom("b"), Atom(<b-val>))))` |
| 5 | Auto-accessors generated per field | `:ns::T/a` and `:ns::T/b` defns auto-generated; each accepts `:ns::T` instance; returns the field's value (HolonAST or typed primitive — sonnet picks + documents) |
| 6 | Predicate unchanged from 227.1b | `:ns::is-T?` works identically; classifier-dispatch via arc 226 `:wat::holon::is?` |
| 7 | Cross-namespace independence holds | `:appA::Voltage [magnitude <- :f64]` and `:appB::Voltage [magnitude <- :f64]` produce distinct classifiers ("appA::Voltage" vs "appB::Voltage"); predicates discriminate |
| 8 | Constructor type-checks each field | Passing wrong-typed arg to constructor → check error citing the field name |
| 9 | Empty field-list `[]` decision documented | Behavior chosen + documented in SCORE (error OR treat-as-single-data; sonnet picks honest path) |
| 10 | New test file `probe_arc227_stone2_defrecord_multifield.rs` | 8+ tests covering construct + accessor read + predicate + cross-namespace + type-check + backward-compat verification; all PASS |
| 11 | All test suites green + holon-rs untouched | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5]` PASS; new probe + 227 stone1 + arc 226 + arc 216 + arc 221 + arc 143 + mvp PASS; wat-edn PASS; clippy clean; holon-rs untouched |
| 12 | SCORE doc written | `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2.md` mirrors SCORE-STONE-227.1b shape; documents accessor-return-type + empty-list decisions |

## Independent prediction (calibration record)

**Target runtime:** 60-120 min Mode A
**Upper bound:** 180 min
**Confidence:** medium-high

**Rationale:**
- Macro extension is real work (field-list parsing + N-arg synthesis + N accessor synthesis) — bigger than 227.1b rename (~5 min) but smaller than 227.1 v3 original mint (~18 min)
- Pattern is locked from defservice precedent (computed unquote + keyword/of + Bundle/children + Option/expect all available)
- Test pattern is mechanical: construct + access + verify

**Risks:**
- Variadic defmacro dispatch by arity — may need investigation; the existing macro is fixed-arity
- Accessor body for "walk Bundle and find matching field" — may need `Bundle/children` iteration + per-item classifier-match; if not ergonomic in pure wat, STOP-5b fires
- Empty field-list edge case — design choice; document honestly
- Accessor return type choice (HolonAST vs typed primitive) — significant ergonomics question

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Methods bundled in defrecord (STOP-6; methods stay separate defns)
- Inheritance via classifier-chain (Stone 227.3)
- `:with-<field>` immutable setters (future stone if requested)
- `:invariants` (future enhancement)
- defprotocol / extend-type (arc 232)
- from-holon support for multi-field structs (future stone)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Honesty deltas accepted

- Variadic defmacro shape may differ from sketch (sonnet picks what wat supports cleanly)
- Accessor return type (HolonAST vs primitive) — sonnet picks + documents in SCORE; either is honest
- Empty field-list behavior — sonnet picks + documents in SCORE
- Test file naming may vary (sibling probe file OR extension of existing probe)
- Number of tests may exceed 8 if sonnet finds more edge cases worth covering

## Honesty deltas NOT accepted

- Breaking Stone 227.1b's single-data form — STOP-8; all 18 existing probe tests must still pass
- Methods bundled in defrecord — STOP-6; `STONE-227.2-NOTES.md` Pattern 3 is doctrine
- Substrate-primitive minting in this stone — STOP-5; if needed, STOP-5b fires (surface as finding)
- Touching holon-rs — STOP-4
- Aliases for the field-list syntax — HARD CUT

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond new probe
- **STOP-3:** 180 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** substrate-primitive route instead of macro
- **STOP-5b:** substrate lacks ergonomic Bundle-walking — surface as finding
- **STOP-6:** methods bundled (Pattern 3 doctrine violation)
- **STOP-7:** bash discipline — cargo hang from pipes
- **STOP-8:** backward compat broken (Stone 227.1b's single-data form stops working)
