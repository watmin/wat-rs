# SCORE — `insert` reports `insert`

Live, user-visible. The checker already took `op`; the 3+-arity `insert` entry fed it a constant
`insert-all`. Threaded the verb from each caller. Two arms, two mutations. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ driven before/after | **HOLD.** Before: `:op` `":wat::rete::insert-all"` on a program that wrote `insert`. After: `:op` `":wat::rete::insert"`. Quoted below. |
| 2 ★ two arms | **HOLD.** `insert_reports_insert` and `insert_all_reports_insert_all` each assert the structured `:op` field, not the rendered message. |
| 3 ★ two mutations | **HOLD.** Re-hardcode each caller; each REDs its own arm. Quoted below. Restored. |
| 4 ★ the const is gone | **HOLD.** `insert_facts_on_session` has no `const OP`. Entry-point consts remain (`eval_insert_public` `insert`, `eval_insert_all_native` `insert-all`) and are what get passed. |
| 5 `insert-all` unchanged | **HOLD.** Its arm still reports `insert-all`. Mutation 2 is the proof it can fail the other way. |
| 6 floor | **HOLD.** `.floor/2026-09-07T05-53-46Z/`: `Summary [ 461.212s] 5472 tests run: 5472 passed (3 slow), 22 skipped`. 5470→5472: the two probe tests. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is the one that matters**: one mutation cannot distinguish "threaded" from "hardcoded to the other constant".

## Row 1 — before / after

`insert_reports_insert` at HEAD, before the thread:

```
assertion `left == right` failed
  left: ":wat::rete::insert-all"
 right: ":wat::rete::insert"
```

After: both arms pass. `insert_all_reports_insert_all` passed before the thread too — that is how the defect survived a single-arm gate.

## Row 3 — two mutations

**Re-hardcode `eval_insert_public`'s call to `":wat::rete::insert-all"`:**

```
assertion `left == right` failed
  left: ":wat::rete::insert-all"
 right: ":wat::rete::insert"
```

Arm: `insert_reports_insert`. Restored.

**Re-hardcode `eval_insert_all_native`'s call to `":wat::rete::insert"`:**

```
assertion `left == right` failed
  left: ":wat::rete::insert"
 right: ":wat::rete::insert-all"
```

Arm: `insert_all_reports_insert_all`. Restored.

Same equality, opposite side.

## The insert-all fixture

A typed `(:wat::rete::insert-all s (PersistentVector record i64))` is a **check-time** refusal
(`PersistentVector` infers a homogeneous element type). That is not the runtime TypeMismatch this
strike names. The insert-all arm therefore `apply`s the public verb with a PersistentVector of i64
so `require_record_fact` is what fires. The 3+-arity `insert` arm is a direct call — that path does
not type-check rest elements as Record, which is why DESIGN's driven form reached runtime.

## What this did not do

Did not touch conferre L2-1 or L2-3. Did not change arity or acceptance. Did not look up "what the
user wrote" at runtime — each entry passes a `'static` constant it already knew.

## Final floor

`.floor/2026-09-07T05-53-46Z/`: `Summary [ 461.212s] 5472 tests run: 5472 passed (3 slow), 22 skipped`.
