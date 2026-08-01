# EXPECTATIONS — insert-all (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate | `-E 'test(/insert_all/)'` | 4/4 — equivalence, oracle==prime, non-vacuity, 2-ary untouched |
| 2 | nothing moved | `-E 'binary_id(wat::rete)'` | all pass; `insert`'s 2-ary signature is preserved so this should be trivially green |
| 3 | the floor | `cargo nextest run --release` | 4215 + the new gate's tests |
| 4 | lint | `cargo clippy --all-targets --release` | silent |
| 5 | the win | seed 40,000 facts, `insert-all` vs `foldl`+`insert` | ~41 ms drop |

## Independent prediction

- **Runtime:** 25–40 min. Three wat forms, one native fn, one dispatch arm, one probe pair. The only
  unfamiliar bit is the multi-arity clause, and `:wat::core::+` is a worked exemplar.
- **Diff size:** ~+120 / −5.
- **The win:** ~41 ms on a 40,000-fact seed — a subtraction of a measured 1027 ns/fact, not a model.
  It will NOT move any grid number: the grid times `:native-ns` (fire only) and seeding is outside it.
  **Say that plainly rather than letting a green grid imply this did nothing.**

## Trap-doors named in advance

- **Row 3 could hide a no-op.** If `insert-all` returned the session unchanged, rows 1's equivalence
  and 2's suite would both pass against an empty fact vector. Assertion 3 (N > 1, `facts` length == N)
  is the only thing that catches it. Treat it as a real gate.
- **STOP-1 is the silent failure.** Routing the 2-ary path through `insert-all` keeps every test green
  and taxes the streaming case — the exact regime the chaos engine lives in. Nothing in the scorecard
  would notice. Verify by reading the emitted form, not by a passing test.
- **Positional `facts` resolution** (STOP-2) also stays green today and writes the wrong slot after any
  future field reorder. Read the lookup; don't trust the row.
- **This stone does not touch fire.** If a rider reports a grid improvement, be suspicious — it means
  something moved that this stone should not have moved.
- **`insert-spec` must not be deleted.** The oracle stays; `insert-all-spec` joins it.

## What would make me reject the strike outright

The 2-ary `insert` re-routed; `facts` resolved positionally; a macro used in place of the multi-arity
`defn` without STOP-3 being reported; or `insert-spec` removed.
