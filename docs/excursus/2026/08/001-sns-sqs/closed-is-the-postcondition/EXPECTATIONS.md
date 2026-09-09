# EXPECTATIONS — closed is the postcondition, not rolled-back

Written before the strike. Graded by my own re-run of every row, never by reading the SCORE.

⛔ = a GATE: an invariant this stone controls. A miss is a failed strike.
▪ = a REPORT: an observation. It is recorded, never gated — gating a consequence has fired
wrongly three times in this arc.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the commit door is closed** | new cell C in `probe_arc278_txn_must_close.wat` | `begin2=Ok` after a `COMMIT` that returned the FK constraint error |
| 2 | ★★ **the original cause survives the commit door** | same cell | the reported error is the **constraint** from `COMMIT`, never `Ok` and never a rollback error |
| 3 | ★★ **an already-closed transaction is not a failure** | new cell D | `autocommit?` is `true` after a clean `commit` **and** after a `rollback`; `close-then-err` on that state returns the original `Err`, does not assert |
| 4 | **door A's original path still holds** | existing cells: `put1`/`put2`/`begin2`/`rollback` | unchanged — `put1=put2=Fatal:no such table: main`, `begin2=Ok`, `rollback=Ok` |
| 5 | **the predicate exists at both layers** | `grep -n is_autocommit src/rust_deps/sqlite.rs wat/sqlite.wat` | one hit each; the wat wrapper has **no `match`** |
| 6 | **the census is still two** | `grep -rn "sqlite::begin" --include=*.wat .` (minus target/) | `sqlite-store.wat:371`, `:387` are the only production sites, and **both** close on **all three** paths |
| 7 | ⛔ **the gate is not ignored** | `grep -n ignore tests/services/probe_arc278_txn_must_close.rs` | no `#[ignore]`; `skipped` does **not** rise above 22 |
| 8 | ⛔ **no loose assertions** | the floor's `no_loose_string_assert` lint | green — `assert_eq!` on parsed fields, not `.contains` |
| 9 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ the run count is reported, not gated — new cells may add one test or two |
| 10 | ⛔ **delivery is still exact** | `circuit.wat` ×5 | `total=8000; distinct=8000` every run |
| 11 | **blast radius** | `git status --porcelain` | only the five briefed files; **no `sqs.wat`, no `mem-store`, no new error variant** |

## REPORTS — recorded, not gated

| ▪ | what | why it is not a gate |
|---|---|---|
| a | `publish + drain` median over 5 | a close that never fires adds no hot-path work; the last stone measured +1.0 %, inside the band. A band is not an invariant |
| b | `dup` per run | expected 0, but `distinct` is the invariant — `dup` is an observation about redelivery |
| c | floor test count | rises by however many cells land; gating a number would go red on a **better** test |
| d | `setup` / `stop` | unrelated to this stone; tracked because they are 40 % of the non-publish wall |

## RUNTIME

25–40 min. The Rust and wat additions are each a handful of lines with an exact exemplar; the
work is in the gate's two new cells and in getting the commit failure to reproduce inside the
probe's existing harness.

## TRAP DOORS

- ⚠ **The bare `bool` was proven on `:rust::cache::Lru::is_empty`, not on `Connection`.** That
  impl block carries `scope = "thread_owned"` and `cache.rs:60` carries `type_params = "K,V"` as
  well — if the macro treats a non-parametric thread-owned bool differently, STOP-1 fires. This
  is the one assumption the probe covers only by analogy.
- ⚠ **Deferred FK needs `PRAGMA foreign_keys=ON` on the same connection**, and the store sets its
  own pragmas at `:345-348`. Cell C must build its own connection, as the scratch probe does —
  it must not try to provoke this through `Store/put`, whose schema has no foreign key.
- ⚠ **`close-then-err` is called on a path where the transaction is open and on one where it may
  not be.** If the reshape accidentally makes the assert unreachable in *both* cases, door B is
  not fixed — it is hidden. Cell D exists to prove the assert still has a live condition.
- ⚠ **A rename touches every call site.** `rollback-then-err` → `close-then-err` is two call
  sites today; if it turns out to be more, that is a census I got wrong and it should be said.
