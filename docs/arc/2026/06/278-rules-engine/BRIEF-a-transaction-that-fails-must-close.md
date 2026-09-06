# BRIEF — a transaction that fails must close

Add `ROLLBACK` at all three layers and call it on the store's error paths, so one failed
statement stops poisoning the connection.

Read `DESIGN-a-transaction-that-fails-must-close.md` first.

## READ IN ORDER

| room | why |
|---|---|
| `src/rust_deps/sqlite.rs:313-318` | **`commit`** — `execute_batch("COMMIT")` + `fault_from_rusqlite`. The exemplar; `rollback` is the same three lines with `"ROLLBACK"` |
| the enclosing `#[wat_dispatch(path = ":rust::sqlite::Connection", scope = "thread_owned")]` | a sibling method registers itself — no separate registration step |
| `wat/sqlite.wat:139-143` | **`:wat::sqlite::commit`** — the wrapper to mirror, including `classify :commit raw` → use `:rollback` |
| `wat/sqlite.wat:76-82` | `classify` takes the op keyword; confirm `:rollback` needs no new arm |
| `wat/query/sqlite-store.wat:343-355` | `put` — the `Err e` arm that must roll back first |
| `wat/query/sqlite-store.wat:356-372` | `delete` — the same shape |
| `wat-scripts/scratch-pad/probe-transient-stop6-open-txn.wat` | the measurement that found this. **It becomes a floor test** |

## THE CHANGE

```
put-rows / delete-rows  →  Err e  →  rollback conn
                                       Ok  → return the ORIGINAL Err e
                                       Err → assert, naming BOTH the original cause
                                             and the rollback failure
```

⚠ **Return the original error, not the rollback's outcome.** The caller needs the cause of its
own failure.

## BLAST RADIUS

`src/rust_deps/sqlite.rs`, `wat/sqlite.wat`, `wat/query/sqlite-store.wat`, and the test file the
probe becomes. **No `sqs.wat`, no `circuit.wat`, no codemod.**

⚠ `wat/` is frozen at build time — rebuild before measuring.

## STOP TRIGGERS

- **STOP-1** — if `#[wat_dispatch]` does not pick the new method up automatically, STOP and
  report what registration is actually required. Do not hand-wire it into a dispatch table
  unbriefed.
- **STOP-2** — if `classify` needs a new arm for `:rollback`, that is fine; **if it needs a new
  error *variant*, STOP** — that is a surface change and this stone does not make one.
- **STOP-3** — **do not add the `:Transient` retry.** It is the next stone, and mixing them
  makes neither measurable.
- **STOP-4** — if rolling back changes what a *successful* put/delete returns in any way, STOP.
  The happy path must be byte-identical in behaviour.
- **STOP-5** — nothing outside the four files.

## PRIOR RESULT TO COPY

`SCORE-transient-means-try-again.md` — the STOP that found this, with the exact measurement row 1
must flip.
