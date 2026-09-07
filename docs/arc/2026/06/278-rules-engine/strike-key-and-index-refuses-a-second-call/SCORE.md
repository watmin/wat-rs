# SCORE — the first-keying door refuses a second keying

Hygiene on an **unreachable** branch. One `debug_assert`, one corrected doc, one debug-gated
`#[should_panic]` carrying `rune:excusare(no-falsifier)`. Floor unchanged. Not a live defect.

## Scorecard

| # | result |
|---|---|
| 1 ★ the guard is wired | **HOLD.** Debug run panics at `session.rs:268`. Quoted below. |
| 2 ★ the rune carries its reason | **HOLD.** `rune:excusare(no-falsifier)` on the test: a `#[should_panic]` on the release floor cannot fail this guard because `debug_assert` compiles out under `--release` and the test is `cfg(debug_assertions)` so it does not exist in the floor binary. |
| 3 ★ severity honest | **HOLD.** Both production callers gate on `is_keyed` AND reuse the stored list. A second call cannot happen, and even if it did the lists could not disagree. Hygiene, not a live defect. |
| 4 ★ no release behaviour | **HOLD.** `.floor/2026-09-07T05-05-09Z/`: `Summary [ 461.287s] 5470 tests run: 5470 passed (2 slow), 22 skipped`. Unchanged. |
| 5 doc matches the code | **HOLD.** "A later call does not replace keys" is gone. The door now says a second call is a caller bug and `writer()` is the post-keying route. |
| 6 still two callers | **HOLD on production.** `fire/mod.rs:885` and `hash_join.rs:282`. The extra two hits are the debug probe calling twice — that is the test, not a third production writer. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the only proof available**, and row 2 is what makes its absence from the floor a stated limit rather than a hole. Row 3 is what stops the type reading as a correctness fix it is not.

## Row 1 — debug panic

`cargo nextest run --no-capture -E 'test(second_key_and_index_on_one_join_panics)'` (debug profile):

```
thread 'rete::kernel::session::join_left_index_tests::second_key_and_index_on_one_join_panics' (1967002) panicked at src/rete/kernel/session.rs:268:9:
key_and_index is the FIRST-keying door and join 1 is already keyed — add tokens through `writer()`
test rete::kernel::session::join_left_index_tests::second_key_and_index_on_one_join_panics - should panic ... ok
```

The release floor cannot run this test and cannot fail on this guard. That is the rune's reason, not a gap. `assert!` was not reached for.

## Why the branch is unreachable

Caller 1 (`keyed_join_persistent`, `fire/mod.rs:883-901`): `if !idx.left_idx.is_keyed(join_id)` then `key_and_index`, else `writer()`. Keys reused via `match idx.left_idx.keys(join_id)`.

Caller 2 (`hash_join_delta`, `hash_join.rs:117-141, 282`): `first_keying = !left_idx.is_keyed(*child_id)`; the `else` arm reads the stored list back with `keys(*child_id).expect("keyed join has keys")`.

`or_insert` stays. On the assert's success path it is a no-op insert of a vacant entry.

## What this did not do

Did not pay `assert!` for a floor-provable mutation. Did not change `or_insert` to `insert`. Did not touch `writer()` or `is_keyed`. Did not dress hygiene as a live defect.

## Final floor

`.floor/2026-09-07T05-05-09Z/`: `Summary [ 461.287s] 5470 tests run: 5470 passed (2 slow), 22 skipped`.
