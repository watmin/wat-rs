# DESIGN — a transaction that fails must close

**The missing `ROLLBACK`.** `src/rust_deps/sqlite.rs` + `wat/sqlite.wat` +
`wat/query/sqlite-store.wat` + one test. Resilience. Prerequisite to the retry stone.

## WHY — measured, and it wedges the connection

`sqlite-store.wat:343-355`, `put` (and `delete` identically):

```wat
(:wat::core::match (:wat::sqlite::begin conn)
  ((:wat::core::Ok _)
    (:wat::core::match (:wat::query::put-rows conn names new-rows)
      ((:wat::core::Err e) (:wat::core::Err e))       ;; ← transaction left OPEN
      ((:wat::core::Ok _) (:wat::sqlite::commit conn)))))
```

`probe-transient-stop6-open-txn.wat`, reproduced by me ×2:

```
begin1=Ok  fail=Fatal:no such table  begin2=Fatal: cannot start a transaction within a transaction  commit=Ok
put1=Fatal                            put2=Fatal: cannot start a transaction within a transaction
```

★ **One failed statement poisons the connection for every operation that follows.** The store
does not die — it keeps answering, wrongly, with *"cannot start a transaction within a
transaction"* for a caller who has done nothing wrong.

★★ **`commit=Ok` proves the transaction was still closeable.** This is not a sqlite limitation.
It is a call that was never made, and the verb to make it does not exist.

## THE VERB IS ABSENT AT BOTH LAYERS

| layer | `begin` | `commit` | `rollback` |
|---|---|---|---|
| `wat/sqlite.wat` | `:132` | `:139` | ⛔ |
| `src/rust_deps/sqlite.rs` | `execute_batch("BEGIN")` | `:314` `execute_batch("COMMIT")` | ⛔ |

★ Each layer has an exact exemplar three lines away. The Rust impl block carries
`#[wat_dispatch(path = ":rust::sqlite::Connection", scope = "thread_owned")]`, so a sibling
method registers itself.

## ⛔ THE ONE CONTRACT DECISION

**Roll back, then return the ORIGINAL error.**

The caller needs to know *why the put failed* — not that the cleanup succeeded. A rollback that
overwrote the cause with its own success would be the same information loss this arc keeps
closing, pointed inward.

★★ **If the rollback itself fails, that is fatal and must say so.** A connection with a
transaction that cannot be closed can serve nothing: every later `begin` fails, and continuing
would answer callers with a lie about their own request. ⚠ That is **not** the `:Transient`
pattern the next stone fixes — `:Transient` is recoverable and dying on it is wrong; an
unclosable transaction is genuinely unrecoverable and dying on it is honest.

## ⛔ THE PROBE BECOMES A GATE

`probe-transient-stop6-open-txn.wat` is a scratch-pad probe today. **It must become a floor
test**, not `#[ignore]`d — it is deterministic, needs no chaos, and it is the only thing that
would catch this regressing.

★ The measurement that found the bug is the measurement that keeps it fixed. Leaving it in
`scratch-pad/` means the next person to touch the error path re-discovers this by hand.

## FILES

`src/rust_deps/sqlite.rs`, `wat/sqlite.wat`, `wat/query/sqlite-store.wat`, and one test.

## OUT OF SCOPE = REJECTED

- **The `:Transient` retry.** This stone unblocks it; it is the next one.
- **The three `_` arms in `sqs.wat`.** Still there, still speaking for four outcomes.
- **Rollback anywhere but the sqlite store's own error paths.** `mem-store` has no transactions.
- Per-entry outcomes, the topic batch, `setup`/`stop`, compiled wat.
