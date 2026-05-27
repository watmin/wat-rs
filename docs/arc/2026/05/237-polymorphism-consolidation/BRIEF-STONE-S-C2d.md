# BRIEF — Stone S-C.2d — mint `:wat::Record/same-data?` (type-blind record data equality)

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (`pwd` first; reject `.claude/worktrees/`; `git -C` if needed).
**Sub-DESIGN:** `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-C2d.md` — read it.
**Mirror:** `eval_record_assoc` (dispatch + eval fn + scheme registration) and `eval_record_to_map`.

## What to do

Mint substrate primitive `:wat::Record/same-data?` — compares two records' field DATA, TYPE-BLIND
and flavor-blind (the user's split: `=` is type-strict, `same-data?` ignores type). It is the
proven composition `(:wat::core::= (:wat::core::record->map a) (:wat::core::record->map b))` —
name-keyed, type-blind — promoted to a named verb.

The FM-2-bis probe is on disk: `tests/probe_arc237_sC2d_same_data.rs`. Its `comp_*` tests
(3, GREEN now) prove the composition; its `samedata_*` tests (3, RED now) are your target. Make
all 6 green.

1. **Dispatch arm** (next to `:wat::Record/assoc`, ~`src/runtime.rs:5344`):
   `":wat::Record/same-data?" => eval_record_same_data(args, list_span, env, sym),`
2. **Eval fn** `eval_record_same_data` — arity 2; eval both args; get each record's field-name→value
   map; compare the two maps. The cleanest impl: factor the body of `eval_record_to_map`
   (`runtime.rs:16711`) into a reusable helper `record_field_map(v: &Value, sym, span) -> Result<Value, RuntimeError>`
   (returns the `HashMap` Value), call it on both args, then
   `Ok(Value::bool(values_equal(&map_a, &map_b) == Some(true)))`. `values_equal` on two HashMaps is
   total post-arc-238 (always `Some`). Keep `eval_record_to_map`'s observable behavior identical
   (it now calls the helper) — its probes must stay green.
3. **Checker scheme** — register `[a <- :wat::Record  b <- :wat::Record] -> :wat::core::bool`
   wherever `:wat::Record/assoc`'s scheme is registered (mirror it). `:wat::Record` is the umbrella
   that accepts any record (base or holonic, any class).

## STOP triggers (REJECTION)

1. Making `same-data?` type-AWARE (it is type-BLIND — cross-type same-named-fields → true).
2. Positional comparison (must be name-keyed via `record->map`; different field names → false).
3. Re-implementing map equality instead of reusing `values_equal`.
4. Breaking `eval_record_to_map` behavior in the refactor (its probes must stay green).
5. Touching holon-rs or `values_equal`'s existing arms.
6. A non-obvious error → STOP + surface verbatim.
7. 45 min (STOP-3); 60 (STOP-4).

## Regression suite

```
cargo build --release -p wat                                          # 0 errors
cargo test --release --test probe_arc237_sC2d_same_data               # 6/6 (comp_* stay green; samedata_* now green)
cargo test --release --lib -p wat                                     # >= 834, 0 failed
cargo test --release --test probe_arc238_eq_completeness              # 8/8 (= unaffected)
cargo test --release --test probe_arc234_stone3a_record_to_map 2>/dev/null || cargo test --release --test probe_arc227_stone2_defrecord  # record->map / defrecord regression green
```
(If a `record->map` probe exists by another name, run it; else the defrecord surface covers it.)

## SCORE doc

`SCORE-STONE-S-C2d.md` (NEW). Scorecard + the dispatch arm + eval fn + helper refactor + scheme
registration + honest deltas + `git status --short`. DO NOT commit.

## Calibration

One primitive mirroring assoc + a helper refactor of eval_record_to_map. **Target band: 25–45 min
Mode A; 45 STOP-3; 60 STOP-4.**
