# SCORE — transient means try again

**STOPPED on STOP-6.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
No production change. `sqs.wat` is byte-identical to `401fd7fd6`. One new file:
`wat-scripts/scratch-pad/probe-transient-stop6-open-txn.wat`.

```
Summary [ 375.034s] 5215 tests run: 5215 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T02-14-04Z/`

A subsequent `put` on the same sqlite connection, after a failed `put`, does **not**
succeed. The rollback is a prerequisite. The retry was not added.

## STOP-6 — the measurement

The mid-strike amendment: sqlite `put` is `begin` → `put-rows` → `commit`, and on
`put-rows` `Err` the transaction is left open. `grep rollback` over
`wat/query/sqlite-store.wat`, `wat/sqlite*.wat`, and `src/intrinsic/sqlite*` is
empty — the verb does not exist to call.

Two cells, one run:

```
CONN=begin1=Ok;fail=Fatal:1:no such table: nosuch;begin2=Fatal:1:cannot start a transaction within a transaction;commit=Ok
STORE=put1=Fatal:no such table: main;put2=Fatal:cannot start a transaction within a transaction
```

**Cell CONN** — raw connection, the mechanism. `begin` succeeds. A statement inside
the transaction fails. The next `begin` is `Fatal: cannot start a transaction within
a transaction`. A later `commit` is `Ok` — the original transaction was still open
and could be closed. The store's error path never closes it.

**Cell STORE** — the retry's actual call. sqlite-store, no schema, so `put` does
`begin` then `DELETE FROM main` and fails. The next `Store/put` on the same peer is
`Fatal: cannot start a transaction within a transaction`.

★ A retry of `put` after `:Transient` (or any `put-rows` `Err`) is a second `begin`
on a connection whose transaction was never closed. It cannot succeed. Adding that
retry would turn "busy, try again" into a wedged queue.

## WHAT WAS NOT DONE

No retry in `sqs.wat`. The three `_` arms still speak for four outcomes. That is
the original defect, and it is still there — **behind** a sqlite-surface hole this
stone was forbidden to grow into.

An in-progress send-path retry was reverted (`git checkout -- wat-scripts/queue/sqs.wat`)
the moment STOP-6 fired. Nothing of it remains.

## THE ROWS

| # | row | result |
|---|---|---|
| STOP-6 | subsequent put after failed put | **does not succeed.** `put2=Fatal: cannot start a transaction within a transaction` |
| 1 | transient no longer kills the queue | not reached |
| 2 | exhausted budget names itself | not reached |
| 3 | Constraint / Fatal die by name | not reached |
| 4 | no `_` on a store response | not reached — `sqs.wat` unchanged |
| 5 | floor | ✅ `Summary [ 375.034s] 5215 tests run: 5215 passed (6 slow), 22 skipped` |
| 6 | publish+drain | not reached — no retry to cost |
| 7 | blast radius | ✅ probe only; `sqs.wat` untouched |

## WHAT THE NEXT STONE MUST BE

**A rollback verb, then the retry.** Not the reverse.

`wat/sqlite.wat` has `begin` and `commit` and no `rollback`. sqlite-store's `put` /
`delete` must close the transaction on `put-rows` / `delete-rows` `Err` *before* a
queue retry is honest. That is a sqlite-surface change. The amendment said: if a
rollback must be added to make the retry safe, that is **its own stone**.

Side observation, not a substitute: after the failed inner statement, `commit` on
the still-open transaction returned `Ok`. That is not rollback, and the store never
calls it on the error path. Do not "fix" the wedge by committing a failed
transaction from the queue.

## NOT TOUCHED

`sqs.wat`. `wat/`. `src/`. No retry. No `Failed[]`. No per-entry outcomes.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-06

**STOPPED, correctly. This is a successful strike, not a failed one.**

```
mine  CONN=begin1=Ok;fail=Fatal:no such table;begin2=Fatal:cannot start a transaction
      within a transaction;commit=Ok
      STORE=put1=Fatal;put2=Fatal:cannot start a transaction within a transaction   (x2 identical)
      sqs.wat byte-identical to 401fd7fd6 — the in-progress retry was reverted cleanly
      floor Summary [ 376.225s] 5215 passed, 22 skipped, 0 FAIL
                                        .floor/2026-09-06T02-21-23Z/
```

★ **STOP-6 was added mid-strike and it is the only reason a retry was not shipped into a wedged
connection.** The amendment came from the builder pushing on *"don't double enqueue out of
laziness"* — chasing that objection is what surfaced the open transaction. **The objection was
about duplicates and it found a wedge.**

★★ **`commit=Ok` is the load-bearing detail.** After the failed statement the transaction was
still open **and could still be closed** — so this is not a sqlite limitation, it is a missing
call. The fix is reachable.

## THE VERB IS MISSING AT BOTH LAYERS — I CHECKED

| layer | `begin` | `commit` | `rollback` |
|---|---|---|---|
| `wat/sqlite.wat` | `:132` | `:139` | ⛔ **absent** |
| `src/rust_deps/sqlite.rs` | — | `:314` | ⛔ **absent** |

So the prerequisite is a three-layer stone with an exact exemplar at every layer: the intrinsic,
the wat wrapper that mirrors `commit`, and the two store error paths that call it.

## WHAT THIS STOP PRESERVED

The original defect is **untouched and still real** — three `_` arms in `sqs.wat` still speak for
four store outcomes, and `:Transient` still kills the queue. That is correct: it is now known to
sit **behind** a sqlite-surface hole, and fixing it first would have produced a retry that cannot
succeed.

★ An executor that had "just made it work" would have shipped a retry, seen it fail, and either
widened the stone into `src/` unbriefed or tuned the test until it passed. **It reverted and
handed back instead**, with the measurement that proves why.
