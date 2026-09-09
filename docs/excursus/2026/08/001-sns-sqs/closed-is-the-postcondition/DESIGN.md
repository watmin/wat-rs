# DESIGN — closed is the postcondition, not rolled-back

**Two doors the ROLLBACK stone (`105ecf16d`) left standing.** Same three files, plus the gate.
Resilience. Still a prerequisite to the `:Transient` retry stone.

## WHY — both measured, both committed as probes at `4e4941e1a`

### Door A — a failing `COMMIT` leaves it open

`wat/query/sqlite-store.wat:378` and `:392` are the same shape in `put` and `delete`:

```wat
((:wat::core::Ok _) (:wat::sqlite::commit conn))    ;; ← commit Err propagates; nothing closes
```

`wat-scripts/scratch-pad/probe-a-failed-commit-also-leaves-it-open.wat`, my runs ×2:

```
fk=Ok;ddl=Ok/Ok;begin1=Ok;insert=Ok;
commit=Constraint:FOREIGN KEY constraint failed;
begin2=Fatal:cannot start a transaction within a transaction
```

★ **The identical STOP-6 string, one line from the fix that just landed.**

⚠ **Reachability, stated honestly.** Not reachable in `circuit.wat` as configured: every store is
`:memory:` (`:1311 :1329 :1647 :1693 :1726 :1733 :1788 :1838 :1894`), the store's own schema
(`sqlite-store.wat:209`, `:359`) declares no foreign key, and `busy_timeout` appears nowhere in
the tree. It is reachable through `sqlite-store::Record`'s `:path` field — a durable, caller-set
field whose entire purpose is a file — where `SQLITE_BUSY` on `COMMIT` is a documented outcome
and, with no `busy_timeout`, returns immediately. **This is a latent door, not a live one, and
the stone is worth it because it costs one line at each of two sites.**

### Door B — the new helper crashes on a success

`wat-scripts/scratch-pad/probe-rollback-with-no-txn.wat`, my run:

```
never-opened    = Fatal:cannot rollback - no transaction is active
after-commit    = Ok/Ok/Fatal:cannot rollback - no transaction is active
double-rollback = Ok/Ok/Fatal:cannot rollback - no transaction is active
still-usable    = Ok/Ok
```

`rollback-then-err` (`sqlite-store.wat:41`) calls `assertion-failed!` when the rollback returns
`Err`. But **the most likely rollback failure means the transaction is already closed** — the
exact postcondition the helper exists to establish — and `still-usable=Ok/Ok` proves the
connection is fine afterward.

★★ On any statement error that auto-rolls-back (`SQLITE_FULL`, `IOERR`, `NOMEM`), the guard the
last stone added **kills the store on a success-in-disguise**. That is reachable on the path it
already guards, today.

## ⛔ THE ONE CONTRACT DECISION

**The invariant is "no transaction is open afterward" — not "a rollback ran."**

The last stone encoded the *action*. This one encodes the *state*. Everything follows:

- already closed → the postcondition holds → return the original `Err e`. **Not an error.**
- open → roll back → `Ok` → return the original `Err e`
- open → roll back → `Err` → **genuinely unrecoverable**; assert naming both causes

The assert survives, and only now does it mean what `docs/excursus/2026/08/001-sns-sqs/a-transaction-that-fails-must-close/DESIGN.md`
said it meant: *a transaction that cannot be closed can serve nothing.* Today it also fires on
transactions that are already closed.

## THE PREDICATE — and it is proven to cross

`rusqlite 0.31` has `Connection::is_autocommit(&self) -> bool` (`rusqlite-0.31.0/src/lib.rs:1012`).
Absent at both our layers.

★ A **bare `bool`** through `#[wat_dispatch]` is unexercised in `wat/*.wat`, so it was measured
before being briefed — `wat-scripts/scratch-pad/probe-a-bare-bool-crosses-the-intrinsic-boundary.wat`
drives `:rust::cache::Lru::is_empty` (`src/rust_deps/cache.rs:123`, the same shape, already
registered):

```
empty-when-new=true;empty-after-put=false;len=1
```

It crosses, and is usable directly in `:wat::core::if` — no `match`, no `Result`.

## OUT OF SCOPE — REJECTED, not deferred

- **`busy_timeout`.** A retry policy is a different decision from a close discipline; setting one
  here would hide Door A rather than close it.
- **Matching the "no transaction is active" message string.** A predicate exists; a string
  comparison would be a convention where a fact is available.
- **`mem-store`, `sqs.wat`, the three `_` arms, the `:Transient` retry.** Untouched.

## THE LADDER — where this lands, stated plainly

Rung 2. After this stone the two `begin` sites in the tree (`:371`, `:387` — the whole census)
each close on every path. **A third `begin` written next month gets no help**: nothing makes an
unpaired `begin` unrepresentable. `tests/lint/` holds 30 lints, so rung 3 for this class is a
`no_unpaired_begin` lint the repo already has the machinery for — **named here, not deferred
into this stone's scope.**
