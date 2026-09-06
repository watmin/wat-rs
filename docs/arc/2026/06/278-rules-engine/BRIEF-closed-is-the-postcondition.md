# BRIEF — closed is the postcondition, not rolled-back

Add the `is_autocommit` predicate at both layers, reshape `rollback-then-err` into a
`close-then-err` that treats an already-closed transaction as success, and route the
**commit-failure** arm through it as well as the statement-failure arm. Extend the existing floor
gate to cover both doors.

Read `DESIGN-closed-is-the-postcondition.md` first — it carries the two measurements and the one
contract decision.

## READ IN ORDER

1. **`src/rust_deps/sqlite.rs:250`** — the `#[wat_dispatch(path = ":rust::sqlite::Connection",
   scope = "thread_owned")]` impl block. **`:325` `rollback`** is your sibling for placement.
   **`src/rust_deps/cache.rs:123` `is_empty(&self) -> bool`** is your exemplar for a bare-bool
   return through the same macro — copy that shape, not a `Result` shape.
2. **`wat/sqlite.wat:146`** — `:wat::sqlite::rollback`, the wrapper you sit beside. Note every
   verb in this file wraps a `Result` with a `match`; **yours will not**, because the intrinsic
   returns a bare `bool`. `:wat::core::bool` is the annotation (lowercase — `wat/doctest.wat:17`).
3. **`wat/query/sqlite-store.wat:41`** — `rollback-then-err` as it stands, and
   **`:32` `sqlite-error-message`**, its message accessor.
4. **`wat/query/sqlite-store.wat:371-378`** (`put`) and **`:387-392`** (`delete`) — the two
   transaction blocks. These are the whole `begin` census in the tree; there is no third.
5. **`tests/services/probe_arc278_txn_must_close.wat`** and **`.rs`** — the gate you extend.
   **`wat-scripts/scratch-pad/probe-a-failed-commit-also-leaves-it-open.wat`** is the worked
   reference for provoking a commit failure; copy its DDL.

## SKETCH

`src/rust_deps/sqlite.rs`, beside `rollback`:

```rust
/// `:rust::sqlite::Connection::is_autocommit conn` — true iff NO transaction
/// is open. Cannot fail, so a bare `bool` (cf. `cache.rs:123`).
pub fn is_autocommit(&self) -> bool {
    self.conn.is_autocommit()
}
```

`wat/sqlite.wat`, beside `rollback` — no `match`, there is no `Result`:

```wat
(:wat::core::defn :wat::sqlite::autocommit?
  [conn <- :wat::sqlite::Connection] -> :wat::core::bool
  (:rust::sqlite::Connection::is_autocommit conn))
```

`wat/query/sqlite-store.wat` — `rollback-then-err` becomes `close-then-err`:

```wat
;; Establish "no transaction is open", then return the ORIGINAL error. An
;; already-closed transaction IS the postcondition — not a failure.
(:wat::core::defn :wat::query::close-then-err
  [conn <- :wat::sqlite::Connection  e <- :wat::sqlite::Error]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::sqlite::autocommit? conn)
    (:wat::core::Err e)
    (:wat::core::match (:wat::sqlite::rollback conn)
      ((:wat::core::Ok _) (:wat::core::Err e))
      ((:wat::core::Err rb)
        (:wat::kernel::assertion-failed!
          (:wat::core::format
            "sqlite-store: transaction will not close; original={o}; rollback={r}"
            :o (:wat::query::sqlite-error-message e)
            :r (:wat::query::sqlite-error-message rb))
          :wat::core::None :wat::core::None)))))
```

Both transaction blocks, `put` and `delete` identically — the commit arm gains a `match`:

```wat
((:wat::core::Ok _)
  (:wat::core::match (:wat::query::put-rows conn names new-rows)
    ((:wat::core::Err e) (:wat::query::close-then-err conn e))
    ((:wat::core::Ok _)
      (:wat::core::match (:wat::sqlite::commit conn)
        ((:wat::core::Ok _)  (:wat::core::Ok nil))
        ((:wat::core::Err e) (:wat::query::close-then-err conn e))))))
```

Extend `tests/services/probe_arc278_txn_must_close.wat` with two cells and assert them in the
`.rs` with `assert_eq!` on parsed fields (`no_loose_string_assert` rejects `.contains` — that lint
is what went red on the last stone's first floor):

- **cell C** — the commit-failure door: the deferred-FK schema from the scratch probe, then
  `commit` (expect the constraint error) and `begin2`, which **must now be `Ok`**.
- **cell D** — the already-closed case: `begin`, `rollback`, then `close-then-err`'s condition —
  `autocommit?` must be `true` after a clean `commit` and after a `rollback`.

## BLAST RADIUS

`src/rust_deps/sqlite.rs`, `wat/sqlite.wat`, `wat/query/sqlite-store.wat`,
`tests/services/probe_arc278_txn_must_close.{rs,wat}`. **No new error variant. No change to
`sqs.wat`, `mem-store`, the three `_` arms, or the `:Transient` retry.**

## STOP TRIGGERS

- **STOP-1** — `#[wat_dispatch]` does not register a bare-`bool` method, or the wat wrapper will
  not type-check against `:wat::core::bool`. Surface the exact checker error.
- **STOP-2** — `close-then-err` needs an error variant that `:wat::sqlite::Error` does not carry.
- **STOP-3** — cell C's `begin2` is still `Fatal: cannot start a transaction within a transaction`
  after the commit arm is routed through `close-then-err`. That means closing needs something
  beyond `ROLLBACK`; report the measurement.
- **STOP-4** — extending the gate requires touching any file outside the blast radius.

## GRADE AGAINST

`SCORE-a-transaction-that-fails-must-close.md` — same arc, same three files, the shape to copy.

Write `SCORE-closed-is-the-postcondition.md`, then `pulsare_yield kind=scored`.
