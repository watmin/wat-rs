# SCORE — closed is the postcondition, not rolled-back

**STRUCK.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
`src/rust_deps/sqlite.rs` + `wat/sqlite.wat` + `wat/query/sqlite-store.wat` +
`tests/services/probe_arc278_txn_must_close.{rs,wat}`.

```
Summary [ 380.006s] 5216 tests run: 5216 passed (5 slow), 22 skipped
```

`.floor/2026-09-06T03-24-38Z/`

The invariant is "no transaction is open afterward." An already-closed transaction
is success. A failed `COMMIT` is closed the same way a failed statement is.

## THE PREDICATE

`Connection::is_autocommit` — bare `bool`, same shape as `cache.rs:123`.
`#[wat_dispatch]` registered it; STOP-1 did not fire.

`:wat::sqlite::autocommit?` is a one-line wrapper with **no `match`**. Used
directly in `:wat::core::if`.

## close-then-err

`rollback-then-err` renamed. Two production `begin` sites (`sqlite-store.wat:373`
and `:392`); each now closes on **all three** paths: begin-Err (nothing open),
rows-Err, commit-Err. Four `close-then-err` call sites (statement + commit, put
and delete) — the rename census was two; the commit arm added two more.

```
already closed  →  return original Err e
open, rollback Ok  →  return original Err e
open, rollback Err →  assert, naming both causes
```

## THE DOORS

Cell C — deferred FK, `COMMIT` returns the constraint, `close-then-err` returns
the **same** constraint, `c-begin2=Ok`:

```
c-insert=Ok;c-commit=Constraint:19:FOREIGN KEY constraint failed;c-closed=Constraint:19:FOREIGN KEY constraint failed;c-begin2=Ok
```

Cell D — `autocommit?` is `true` after a clean commit and after a rollback;
`close-then-err` returns `Fatal:0:probe-cause` and does not assert:

```
d-ac-commit=true;d-cte-commit=Fatal:0:probe-cause;d-ac-rollback=true;d-cte-rollback=Fatal:0:probe-cause
```

Existing cells unchanged: `put1=put2=Fatal:no such table: main`, `begin2=Ok`,
`rollback=Ok`.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ commit door closed | ✅ `c-begin2=Ok` |
| 2 | ★★ original cause survives commit door | ✅ `c-closed=Constraint:19:FOREIGN KEY constraint failed` |
| 3 | ★★ already-closed is not a failure | ✅ `d-ac-commit=true`, `d-ac-rollback=true`, both `close-then-err` return `probe-cause` |
| 4 | door A's original path | ✅ `put1=put2=Fatal:no such table: main`, `begin2=Ok`, `rollback=Ok` |
| 5 | predicate at both layers | ✅ `sqlite.rs:333`, `wat/sqlite.wat:153`; wrapper has no `match` |
| 6 | census still two production `begin`s | ✅ `sqlite-store.wat:373`, `:392`; both close on all three paths |
| 7 | ⛔ gate not ignored | ✅ no `#[ignore]`; skipped stays 22 |
| 8 | ⛔ no loose assertions | ✅ `assert_eq!` on parsed fields; lint green |
| 9 | ⛔ floor | ✅ `Summary [ 380.006s] 5216 tests run: 5216 passed (5 slow), 22 skipped` |
| 10 | ⛔ delivery exact | ✅ `total=8000; distinct=8000; dup=0` ×5 |
| 11 | blast radius | ✅ the five briefed files; no `sqs.wat`, no new error variant |

## REPORTS

| ▪ | observation |
|---|---|
| a | publish+drain ×5: 23789 23941 23904 23913 23941 — median **23913** (before 23672) |
| b | `dup=0` ×5 |
| c | floor still 5216 — same test, more `assert_eq!` |
| d | setup ~9.7 s, stop ~6.3 s |

## NOT TOUCHED

`sqs.wat`. The three `_` arms. `:Transient` retry. `busy_timeout`. `mem-store`.
No string-match on "no transaction is active".

The retry stone is unblocked: every `begin` in production now closes on every path.

---

Tree uncommitted. Do not commit unless asked.
