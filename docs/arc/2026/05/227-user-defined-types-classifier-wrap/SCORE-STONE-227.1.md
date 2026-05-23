# SCORE — Arc 227 Stone 227.1 — User-defined types via `:wat::holon::defclass` macro

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 12/12 PASS — all deliverables complete, all suites green

| # | Deliverable | Status | Citation |
|---|---|---|---|
| 1 | NEW `:wat::holon::defclass` defmacro authored | PASS | `wat/holon/defclass.wat` — `(:wat::core::defmacro (:wat::holon::defclass (fqdn :AST<wat::core::nil>) -> :AST<wat::core::nil>) ...)` quasiquote body; expands to `(:wat::core::do defn-constructor defn-predicate)`; single-arg form only (stone 227.1) |
| 2 | defmacro registered via stdlib defmacros | PASS | `src/stdlib.rs` — WatSource entry for `wat/holon/defclass.wat` inserted after Trigram.wat; comment cites arc 227 Stone 227.1; auto-loaded via `register_stdlib_defmacros` at startup; verified by tests calling `(:wat::holon::defclass ...)` directly without user import |
| 3 | Constructor auto-generated in USER-DECLARED namespace | PASS | Computed unquote `~fqdn` substitutes the user-declared FQDN keyword directly as the `defn` name; `~(:wat::core::keyword/to-string fqdn)` produces the classifier string literal at expand-time; constructor body = `(:wat::holon::Bind (:wat::holon::Atom (:wat::holon::to-holon "ns::Name")) (:wat::holon::Atom v))`; typed `[v <- :wat::holon::HolonAST]` |
| 4 | Predicate auto-generated in USER-DECLARED namespace | PASS | Second computed unquote `~(:wat::core::let [...] (:wat::core::keyword/from-string ...))` builds predicate FQDN at expand-time: splits FQDN string on `::`, takes all-but-last as namespace prefix, prepends `"is-"` to basename, suffixes `"?"`, calls `keyword/from-string`; predicate body delegates to `(:wat::holon::is? v classifier-string)` |
| 5 | Classifier string = FQDN without leading colon (collision-free) | PASS | `(:wat::core::keyword/to-string fqdn)` strips leading colon automatically; `"myapp::Voltage"` not `"Voltage"`; `:appA::Voltage` vs `:appB::Voltage` → distinct classifiers verified by `probe_defclass_cross_namespace_discrimination` |
| 6 | Multiple namespaces independent | PASS | `probe_defclass_cross_namespace_app_a_positive` + `probe_defclass_cross_namespace_discrimination` + `probe_defclass_cross_namespace_app_b_positive` — appA instance ≠ appB instance; both predicates work independently |
| 7 | User types distinct from built-in types | PASS | `probe_defclass_user_type_vs_builtin_not_map` — `(:test::MyMap ...)` produces classifier "test::MyMap"; `(:wat::holon::is-Map? instance)` → false; user classifier never masquerades as arc 228 built-in |
| 8 | Polymorphic `:wat::holon::is?` works on user types | PASS | `probe_defclass_polymorphic_is_fqdn_positive` → true; `probe_defclass_polymorphic_is_bare_basename_negative` → false; arc 226's `is?` correctly discriminates FQDN-qualified vs bare basename |
| 9 | Constructor enforces HolonAST input at type boundary | PASS | `probe_defclass_constructor_typed_rejects_non_holon` — passing raw `5.0` literal to constructor fails at check time (TypeMismatch/MalformedForm); the `[v <- :wat::holon::HolonAST]` typed parameter enforces boundary at the type checker |
| 10 | New test file `probe_arc227_stone1_defclass.rs` | PASS | 18 tests covering: single defclass positive/negative, cross-namespace appA/B independence, cross-namespace discrimination, same-namespace cross-discrimination, user vs built-in, polymorphic is? FQDN positive + bare-basename negative, constructor type rejection, multi-segment 3-level namespace, predicate name shape, i64 payload, cross-type discrimination, no :user::* insertion, appB independent; all 18 PASS |
| 11 | All test suites green + holon-rs untouched | PASS | See test summary below |
| 12 | No `:user::*` insertion anywhere | PASS | `probe_defclass_no_user_namespace_insertion` — macro generates `:test::is-Celsius?` in `:test::` namespace; `grep` of defclass.wat shows no `:user::` string; all user-declared FQDN keywords flow through as-is |

## Test summary

```
cargo build --release -p wat                                           — 0 errors (5 pre-existing unused-fn warnings)
cargo test --release --lib -p wat [skip 5 signal tests]               — 822/822 PASS
cargo test --release --test probe_arc227_stone1_defclass               — 18/18 PASS
cargo test --release --test probe_arc226_stone1_type_predicates        — 27/27 PASS
cargo test --release --test probe_arc216_stone1_hashset_roundtrip      — 10/10 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip       — 12/12 PASS
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip      — 14/14 PASS
cargo test --release --test probe_arc216_stone4_predicate_composition  — 6/6 PASS
cargo test --release --test probe_arc216_stone7_tuple_roundtrip        — 12/12 PASS
cargo test --release --test wat_arc221_keyword_nil_tag_atomization      — 6/6 PASS
cargo test --release --test wat_arc143_manipulation                    — 8/8 PASS
cargo test --release --test mvp_end_to_end                             — 10/10 PASS
cargo test --release -p wat-edn                                        — 1/1 PASS (doc test)
cargo clippy --release --all-targets -p wat-edn -- -D warnings         — 0 warnings

holon-rs contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only           — empty (untouched)
```

## Deltas from EXPECTATIONS

### Delta 1 — Constructor parameter typed as `:wat::holon::HolonAST` (boundary-honest design)

EXPECTATIONS row 9 described "constructor errors on non-atomizable" with the note "errors at check time per arc 225 narrow Atom". The v1 macro draft used untyped `[v]` which the type checker rejected. The correct form is `[v <- :wat::holon::HolonAST]` — honest: the constructor is a boundary-crossing verb; the user lifts their primitive values via `(:wat::holon::to-holon ...)` before calling the constructor.

This IS the right design: the constructor receives an already-lifted `HolonAST` value and Atom-wraps it. The "non-atomizable" error now triggers at check time when a caller passes a raw primitive (e.g. `5.0`) directly to the constructor — which is the `probe_defclass_constructor_typed_rejects_non_holon` test. This is more honest than a runtime error: the type boundary is enforced at check time.

### Delta 2 — 18 tests instead of minimum 6

EXPECTATIONS row 10 specified "6+ tests". The implementation ships 18 — systematic coverage of all FQDN shapes (2-segment, 3-segment), both namespaces in cross-namespace tests, predicate name shape verification, constructor type enforcement, and no-:user::* insertion. All 18 add signal; none are padding.

### Delta 3 — Computed unquote uses `let` + `keyword/from-string` for predicate FQDN

The BRIEF sketched `classifier-string-from` and `predicate-fqdn-from` as hypothetical helpers. The substrate already has everything needed: `keyword/to-string` + `string::split` + `Vector/length` + `last` + `Option/expect` + `take` + `string::join` + `string::concat` + `keyword/from-string`. The predicate FQDN is built entirely from existing stdlib primitives inside a computed unquote `let` block. No new helpers minted. STOP-5 and STOP-5b did not trigger.

## STOP trigger audit

- **STOP-1 (unexpected substrate compile error):** DID NOT TRIGGER. One type-check iteration: initial macro used untyped `[v]` which the type checker rejected; fixed to `[v <- :wat::holon::HolonAST]` — expected iteration, not unexpected.
- **STOP-2 (test failure beyond new probe):** DID NOT TRIGGER. All 822 lib tests + all arc 216/221/143/226/mvp probes PASS.
- **STOP-3 (180 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched):** DID NOT TRIGGER. `git -C holon-rs/ diff --name-only` empty.
- **STOP-5 (substrate-primitive route):** DID NOT TRIGGER. Pure macro expansion using existing substrate primitives.
- **STOP-5b (substrate lacks keyword-manipulation helpers):** DID NOT TRIGGER. `keyword/to-string`, `keyword/from-string`, `string::split`, `string::join`, `string::concat`, `Vector/length`, `last`, `take`, `Option/expect` all present. No new helpers needed.
- **STOP-6 (inheritance scope creep):** DID NOT TRIGGER. Stone 227.1 ships single-arg `defclass` only.
- **STOP-7 (bash discipline):** DID NOT TRIGGER. One cargo command at a time, foreground.
- **STOP-8 (namespace insertion violation):** DID NOT TRIGGER. The macro uses `~fqdn` for the constructor name and computes the predicate FQDN from the user-declared FQDN — no `:user::*` insertion anywhere.

## Files changed

**wat stdlib (new):**
- `wat/holon/defclass.wat` — `(:wat::core::defmacro (:wat::holon::defclass (fqdn :AST<wat::core::nil>) -> :AST<wat::core::nil>) quasiquote-body)` — 67 lines including doc comment; two computed unquotes for classifier string + predicate FQDN; expands to `(:wat::core::do defn-constructor defn-predicate)` per design

**wat-rs source (Rust — modified):**
- `src/stdlib.rs` — WatSource entry for `wat/holon/defclass.wat` inserted after Trigram.wat with arc 227 Stone 227.1 citation comment; `include_str!("../wat/holon/defclass.wat")` bakes the macro into the binary

**Test files (Rust — new):**
- `tests/probe_arc227_stone1_defclass.rs` — 18 tests covering the full EXPECTATIONS matrix plus edge cases

**Total: 1 new wat stdlib file + 1 modified Rust source + 1 new test file + 1 new SCORE doc.**

## Substrate state post-Stone-227.1

**User-defined types fully operational via classifier-wrap:**
- `(:wat::holon::defclass :myapp::Voltage)` — mints constructor `:myapp::Voltage` and predicate `:myapp::is-Voltage?` in the user-declared namespace
- Constructor: `[v <- :wat::holon::HolonAST] → Bind(Atom("myapp::Voltage"), Atom(v))`
- Predicate: `[v <- :wat::holon::HolonAST] → is?(v, "myapp::Voltage")`
- Classifier string = FQDN without leading colon — collision-free across applications
- Multi-segment namespaces (`:awesome::lib::Sensor`) produce `:awesome::lib::is-Sensor?` correctly
- Polymorphic `:wat::holon::is?` (arc 226) works on all user-defined types

**Typed-entities doctrine chain COMPLETE:**
```
arc 225 ✓ — bridge naming family (substrate verbs honest)
arc 228 ✓ — collection classifier-wrap
arc 230 ✓ — variant retirement (substrate 16 → 12 primitives)
arc 226 ✓ — type predicates (substrate IS the type system)
arc 227 ✓ — user-defined types in USER-DECLARED namespaces  ← THIS STONE
```

The duck has a measurable shape. Users name new ducks in namespaces they own. The substrate stays uninvolved.

## Unblocks

- Arc 227 INSCRIPTION (Stone 227.4) — cascade: arc 227 → arc 226 INSCRIPTION → full chain close
- Arc 227 Stone 227.2 — inheritance via classifier-chain (`(defclass :myapp::U8 :wat::core::Int)`)
- Arc 227 Stone 227.3+ — multimethod dispatch integration with arc 146/147
- Arc 227 Stone 227.4 — USER-GUIDE chapter on user-defined types via classifier-wrap
- Any user-surface code that declares domain types in their own namespace

## Addendum 2026-05-22 night — Stone 227.1b rename (defclass → defrecord)

Per user direction post-ship: defclass renamed to defrecord. Rationale: "class" implies methods + mutable state; "record" is honest about immutable data-only. Locks the honest name before arc 232 (defprotocol) builds on it.

- Macro: `:wat::holon::defclass` → `:wat::holon::defrecord` (HARD CUT — no alias)
- File: `wat/holon/defclass.wat` → `wat/holon/defrecord.wat`
- Probe: `tests/probe_arc227_stone1_defclass.rs` → `tests/probe_arc227_stone1_defrecord.rs`
- Commit: [TBD by orchestrator]

This SCORE doc's body above remains unchanged as historical record per `feedback_inscription_immutable`.
