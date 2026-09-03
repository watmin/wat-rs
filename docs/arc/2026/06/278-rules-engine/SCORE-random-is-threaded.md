# SCORE — random is threaded

**NOT STRUCK.** Executor: grok, 2026-09-03. The verbs landed and classify
apart. The floor went red on a timeout this stone did not touch. The ARM
was kept. **The floor was not re-run.**

```
Summary [ 359.382s] 5200 tests run: 5199 passed (3 slow), 1 timed out, 15 skipped
```

Log: `.floor/2026-09-03T09-14-58Z/`
ARM: `.floor/2026-09-03T09-14-58Z/ARM.txt`

## The red

```
TIMEOUT [  30.015s] (3985/5200) wat::services probe_async_publish::refused_subscriber_is_retried_not_dropped
stdout:
    running 1 test
    (test timed out)
```

No assertion fired. The test did not return in 30s. The function it
drives is `:user::refused-is-retried` in `sns-fanout.wat` — wait-inflight
on the inbox, then wait-pending on the subscriber after a 350 ms nap.

This stone did not edit `sns-fanout.wat`, `circuit.wat`, or `sqs.wat`.
On the previous floor (`.floor/2026-09-03T08-11-03Z/`) the same test was
`PASS [1.624s]`. Sibling probes in the same file that freeze the same
topic file all passed here in 1.0–1.4 s, so freeze is not the hang.

That is the finding. It is not a disposition.

## What landed (rows 1–8)

Two verbs, two classes, two names. Not arity-overloaded.

```
(:wat::rand::int      lo hi)        -> i64                ambient  · Pure, NOT Deterministic
(:wat::rand::int-from state lo hi)  -> (Tuple i64 i64)    threaded · Pure AND Deterministic
```

Both `[lo, hi)`. Tuple is `(new-state, draw)`. SplitMix64, reject-and-redraw.
`int` is a wrapper over `int-from` with a fresh seed from `uuid::v4`'s
entropy. One algorithm. No `Rng` type. State is `i64`. No statics, no
thread-locals, no cells.

`wat/`, `wat-scripts/` empty.

## EXPECTATIONS

| # | what | this strike |
|---|---|---|
| 1 | ★ BOTH axes classify | ✅ `int-from`: Pure ok, Deterministic ok |
| 2 | ★ same seed, same sequence | ✅ 100 draws from S twice, identical |
| 3 | ★ different seeds diverge | ✅ S vs S+1, 100 draws differ |
| 4 | ★ in range and unbiased | ✅ 100k `int-from` on `[0,6)`: every value in range, each bucket within 3% of even |
| 5 | ★ no ambient state | ✅ state is a parameter and a return; `int` seeds per call from uuid entropy, no cell |
| 6 | registered in both places | ✅ `register_builtins` + `intrinsic_meta`; `checker_skip_debt` and `every_dispatched_verb` green in debug |
| 6b | ★ classified DIFFERENTLY | ✅ `int-from` both axes; `int` Pure ok, Deterministic **err** — same as `uuid::v4` |
| 6c | ambient is a wrapper | ✅ `eval_rand_int` calls `int_from(fresh_seed(), lo, hi)` |
| 7 | no new core type | ✅ state is `i64` |
| 8 | no `.wat` corpus change | ✅ `wat/`, `wat-scripts/` empty |
| 9 | floor | ❌ 5199 passed, **1 timed out**, ARM kept, not re-run |

## Files

- `src/intrinsic/rand.rs` — the two verbs and the algorithm tests
- `src/intrinsic/mod.rs` — `mod rand`
- `src/check.rs` — both TypeSchemes
- `src/rete/purity.rs` — `int` with `uuid::v4`; `int-from` in `pure_det`; row 6b test
