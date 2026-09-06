# SCORE — a transaction that fails must close

**STRUCK.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
`src/rust_deps/sqlite.rs` + `wat/sqlite.wat` + `wat/query/sqlite-store.wat` +
`tests/services/probe_arc278_txn_must_close.{rs,wat}`. Scratch-pad STOP-6 probe
removed (it is the floor gate now).

```
Summary [ 383.178s] 5216 tests run: 5216 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T02-50-21Z/`

A failed statement no longer poisons the connection. `put2` reports the original
cause.

## THE VERB, THREE LAYERS

| layer | what |
|---|---|
| `src/rust_deps/sqlite.rs` | `Connection::rollback` — `execute_batch("ROLLBACK")`, sibling of `commit`. `#[wat_dispatch]` picked it up; STOP-1 did not fire |
| `wat/sqlite.wat` | `:wat::sqlite::rollback` — `classify :rollback raw`. No new error variant; STOP-2 did not fire |
| `wat/query/sqlite-store.wat` | `put` and `delete` `Err` arms call `rollback-then-err`: rollback Ok → return the **original** `Err e`; rollback Err → assert naming both causes |

Happy path is still `begin` → rows → `commit`. Rollback is not on that path.

## ★★ ROW 1 — put2 flipped

Before (STOP-6, verbatim):

```
STORE=put1=Fatal:no such table: main;put2=Fatal:cannot start a transaction within a transaction
```

After:

```
begin1=Ok;fail=Fatal:1:no such table: nosuch;rollback=Ok;begin2=Ok;commit=Ok;put1=Fatal:no such table: main;put2=Fatal:no such table: main
```

`put2` is **`Fatal:no such table: main`** — the same original cause as `put1`, not
the poisoned-connection string. CONN `begin2=Ok` after an explicit rollback.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ connection no longer poisoned | ✅ `put2=Fatal:no such table: main` |
| 2 | ★★ original cause survives | ✅ put1 and put2 are the same `no such table: main`; not the rollback's Ok |
| 3 | rollback at all three layers | ✅ rust + wat + both store error paths |
| 4 | ⛔ probe is a floor gate | ✅ `tests/services/probe_arc278_txn_must_close.rs`, not `#[ignore]`d. 5216 tests, 22 skipped |
| 5 | ⛔ floor | ✅ `Summary [ 383.178s] 5216 tests run: 5216 passed (6 slow), 22 skipped` |
| 6 | ⛔ publish+drain ×5 | ✅ median **23905 ms** vs before **23672 ms** (+1.0 %, inside the existing band) |
| 7 | blast radius | ✅ the three sqlite files + the gate; no `sqs.wat`, no retry |

## ⚠ FIRST FLOOR WAS RED — captured, not re-run

`.floor/2026-09-06T02-41-51Z/`

```
Summary [ 383.772s] 5216 tests run: 5215 passed (6 slow), 1 failed, 22 skipped
FAIL wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
tests/services/probe_arc278_txn_must_close.rs:51, :55, :59
```

The gate used `.contains` on the summary. Replaced with `assert_eq!` on `put1` /
`put2` / `begin2` / `rollback`. Second floor is the green Summary above.

## ROW 6 — quiet circuit ×5

`ps` before: grok 9.1 %, claude 5.8 %, else < 1 %.

```
publish        24213  23895  23668  23526  23705     median 23705
drain            210    195    200    199    200     median   200
publish+drain  24423  24090  23868  23725  23905     median 23905
```

Before median **23672**. +233 ms. `total=8000; distinct=8000; dup=0` ×5. A rollback
that never fires is not a new hot-path timer.

## NOT TOUCHED

`sqs.wat`. The three `_` arms. `:Transient` retry (STOP-3). `mem-store`. No new
error variant.

The retry stone is unblocked: a failed sqlite `put` no longer leaves the
transaction open.

---

Tree uncommitted. Do not commit unless asked.
