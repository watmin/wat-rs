# SCORE — Stone S-B.2 — defrecord emits `recordtype` + drops its hand-rolled predicate

**Date:** 2026-05-26
**Status:** COMPLETE — 5/5 PASS (LOAD-BEARING); all regression guards green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -3` | 0 errors |
| 2 | **S-B.2 probe 5/5 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | arc 227 defrecord regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -3` | `35 passed; 0 failed` |
| 5 | arc 234 stone 2b defrecord macro | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | arc 234 stone 3a record read verbs | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | arc 234 stone 3b record assoc | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | arc 234 stone 3c keyword accessor | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | arc 234 stone 4 match hash destructure | `cargo test --release --test probe_arc234_stone4_match_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 10 | arc 234 stone 5 holon auto dispatch | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 11 | arc 237.6 is-predicate regression | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 12 | arc 237 S-B.1 recordtype regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 13 | arc 237 S-A hierarchy regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 14 | holon-rs untouched | (not touched) | STOP-5 not triggered |

---

## The two macro edits (`wat/Record.wat`)

### Edit 1 — ADD `recordtype` as the first form in the `do` block (line 116)

Before:
```wat
  `(:wat::core::do
     (:wat::core::defn ~fqdn [~@fields] -> :wat::Record
```

After:
```wat
  `(:wat::core::do
     (:wat::core::recordtype ~fqdn :wat::Record)
     (:wat::core::defn ~fqdn [~@fields] -> :wat::Record
```

### Edit 2 — REMOVE the hand-emitted predicate `defn` (was the last `do` form, ~lines 220-232)

Removed:
```wat
     (:wat::core::defn ~(:wat::core::let
                           [fqdn-str  (:wat::core::keyword/to-string fqdn)
                            parts     (:wat::core::string::split fqdn-str "::")
                            n         (:wat::core::Vector/length parts)
                            basename  (:wat::core::Option/expect -> :wat::core::String
                                        (:wat::core::last parts)
                                        "Record::def: FQDN must have at least one segment")
                            pfx-parts (:wat::core::take parts (:wat::core::i64::-'2 n 1))
                            pfx-str   (:wat::core::string::join "::" pfx-parts)]
                           (:wat::core::keyword/from-string
                             (:wat::core::string::concat pfx-str "::" "is-" basename "?")))
                       [v <- :wat::Record] -> :wat::core::bool
       (:wat::core::conforms? v ~fqdn))))
```

The `do` block now closes after the accessor splice: `accessors)))`.

---

## Test-expectation updates

**None.** Zero test files needed expectation updates.

The consumer ripple did NOT surface any test asserting the old type-error-on-non-record behavior. All 17+ defrecord test consumers call `is-<Name>?` on record values (TRUE-path or cross-class-false on another record), so the synthesized ∀T form returns the same result as the old narrowing form in those cases. The `probe_arc234_stone2b_defrecord_macro` probe_4 (`predicate_false_on_non_matching_class`) tests cross-class false with two record values — `is-Voltage? (Pressure instance)` → `false`. The synthesized predicate correctly returns `false` via `concrete_type_name_matches` (class fqdn mismatch), matching the old behavior.

`probe_arc237_stone6_is_predicate` (237.6) passed without expectation update — it tests struct/enum/union `is-X?` predicates which are not emitted by the defrecord macro.

---

## Honest deltas

### is-X? now synthesized ∀T on the everyday surface

`(:wat::Record::def :my::Circle [...])` now produces a synthesized `is-Circle?` that accepts ANY value and returns `bool` (∀T). The old macro-emitted form narrowed `[v <- :wat::Record]` and type-errored at the call site when passed a non-record. The asymmetry-kill is now on the EVERYDAY surface, not just via hand-written `recordtype` (S-B.1).

### Constructor return unchanged — stays `-> :wat::Record`

Constructor return remains `-> :wat::Record` per BRIEF discipline. The per-class return type (`-> :my::Circle`) is the S-A1 pairing; it requires arg-boundary subtyping to be wired before the accessors can accept a narrower type. Not touched here.

### TRUE-path now provable on the everyday surface

B.1 had no constructor (recordtype declares a type, mints none), so it couldn't exercise `(:my::is-Circle? (:my::Circle 1.0))` → `true`. B.2 brings the constructor, making the TRUE-path provable: probe_02 confirms it.

### No DuplicateDefine

The macro dropped its own predicate; `register_type_predicates` synthesizes it. Startup succeeds cleanly with both the macro expansion and the synthesized predicate (probe_01 through probe_05 all require startup to succeed — implied no DuplicateDefine).

### Pure wat-macro edit — zero Rust substrate changes

`src/*.rs` untouched. `holon-rs` untouched. B.1 shipped the machinery; B.2 routes the everyday surface onto it. One file changed: `wat/Record.wat`.

---

## Working tree on return

```
 M wat/Record.wat
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-B2.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit — orchestrator commits after scoring.
