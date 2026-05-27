# EXPECTATIONS — Stone S-B.2

Mode A: 5/5 on the probe + clean baseline (or is-X? expectation-shift only) + the
everyday `defrecord` surface emits `recordtype` and drops its hand-rolled predicate,
constructor return unchanged (`:wat::Record`).

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile/startup clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **S-B.2 probe 5/5** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sB2_defrecord_recordtype 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | arc 227 defrecord regression | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -3` | pass (or expectation-shift only) |
| 5 | arc 234 record read/assoc/accessor/match/holon | per-probe (see BRIEF suite) | pass |
| 6 | 237.6 is-predicate | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | pass; if it asserted the macro's old predicate emission, expectation-shift (note in SCORE) |
| 7 | S-B.1 recordtype regression | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | S-A hierarchy regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 9 | is-X? ∀T (asymmetry dead, everyday surface) | probe 1 | `(:my::is-Circle? 42)` → false, not error |
| 10 | is-X? TRUE-path (B.1-deferred) | probe 2 | `(:my::is-Circle? (:my::Circle 1.0))` → true |
| 11 | constructor return unchanged | grep wat/Record.wat | `(:wat::core::defn ~fqdn [~@fields] -> :wat::Record` still present |
| 12 | files in scope | `git status --short` | `wat/Record.wat` (+ any test-expectation updates) + SCORE doc; NO src/*.rs |

**Clippy NOT a ceiling concern** per standing direction.

## Independent prediction

**Target band: 30–60 min Mode A. STOP-3: 80 min. STOP-4 (hard kill): 110 min.**

The macro edit itself is ~2 forms (add recordtype, remove predicate). The variable
is the consumer ripple across ~17 defrecord-using `tests/*.rs`. Most pass unchanged;
a small number may need an is-X? expectation-shift (type-error→false on non-record).
>3 test-file updates → STOP + report.

## Risks / trap-doors

1. **Constructor-return flip** — must STAY `:wat::Record`; flipping to `:my::Circle`
   breaks the accessors (`[v <- :wat::Record]`) until S-A1. (BRIEF STOP-8.)
2. **DuplicateDefine** — if the macro adds recordtype but FAILS to remove its own
   predicate, `register_type_predicates` collides with the macro's `is-X?`. Both
   edits are mandatory together.
3. **is-X? expectation-shift in existing tests** — the asymmetry-kill changes is-X?
   on a non-record from type-error to `false`. Update those test EXPECTATIONS (not
   substrate). >3 files → STOP.
4. **237.6 probe** — it may assert the macro's old predicate-emission shape. If red,
   it's the expected shift (macro no longer emits; synthesized replaces) — update +
   note. NOT a regression.
5. **Holon flavor creep** — parent stays `:wat::Record`; the base/holonic split is S-C.

## SCORE

`SCORE-STONE-S-B2.md` (NEW). 12-row scorecard + the two macro edits + the list of
any test-expectation updates (with why) + honest deltas (incl. is-X? now synthesized
on the everyday surface; constructor return unchanged) + working tree. Mirror
SCORE-STONE-S-B1 shape.
