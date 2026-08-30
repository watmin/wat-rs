# BRIEF — arc 301 stone 2b: the delete differential (mem vs sqlite)

**This stone adds NO production code.** Stone 2 shipped `delete` in both backends; only
mem-store is exercised. 54 of those 114 lines are covered by nothing. 2b closes that.

## ⚠ This stone may legitimately go RED — and that is a RESULT, not a failure

Every other stone here has been "make the red thing green." **This one is a measurement whose
outcome is unknown.** The sqlite `delete` path has never executed. If mem and sqlite disagree,
**the disagreement IS the finding** — report it, do not reach into `sqlite-store.wat` to make
the numbers match. A differential that gets quietly "fixed" has destroyed the only evidence it
was built to produce. See STOP-1.

## The work

Write a differential fixture that runs the SAME delete sequence over `mem-store` (the oracle)
and `sqlite-store` (`:memory:`), and asserts they agree — on the surviving row count, the
surviving keys, AND the surviving GSI projections.

### ★ It MUST declare a secondary index. This is the whole point.

Stone 2's STOP-2 argument was: *a `Key` is sufficient because `clear-index-projections`
deletes `index_<name>` rows by `(pk, sk)`, so no read-before-delete is needed.* That argument
is **entirely about the GSI path**. A differential with `:index-names []` never calls
`clear-index-projections` at all, and would ship a green number that proves nothing about the
claim it exists to check.

So: declare one index, put rows that project into it, delete one, and **`scan-index` afterwards
on both backends**. If a deleted row's projection survives in sqlite's `index_<name>` table,
that is a real orphaned-index bug and exactly what this stone is for.

## Read in order

1. **`tests/services/probe_arc278_journal_backend_differential.wat`** — the template, whole.
   Copy its shape: a helper parameterized on `store-addr`, run against both backends, compare,
   return a sentinel string on mismatch that the `.rs` catches. Note `sqlite-store/start` takes
   `:record (... :path ":memory:" :index-names [...])` — no temp file, no cleanup.
2. **`tests/rete/probe_arc301_store_delete.wat`** — stone 2's mem probe. Its `three-rows`,
   `ensure-schema`, `put`, `scan-count` and `delete-b` helpers are the sequence to generalize;
   they currently hard-code `mem-store` at the call site only.
3. **`wat/query.wat:26`** — `IndexKey` (`ipk`/`isk`), and `StoredRow.index-keys` at `:34`
   (`index-name -> IndexKey`). This is how a row declares its projection.
4. **`wat/query/sqlite-store.wat:155`** — `clear-index-projections`, the code under test.
5. **`docs/arc/2026/08/301-sns-sqs/SCORE-stone-2-store-delete.md`** — findings 1 and 2, which
   this stone closes.

## Also close finding 2 — the duplicate ack

SQS acks are at-least-once, so **deleting a key that is not present is normal traffic, not an
edge case.** Stone 2 reported `:Success` for it and did not invent a `NotFound` arm — correct,
but untested. Add it as a row: delete the same key twice, assert both backends return
`:Success` and the second is a no-op.

## Implementation sketch — you fill it

```wat
;; one roundtrip, parameterized on the backend's Address'
(:wat::core::defn :user::delete-roundtrip
  [store-addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String                     ;; an EDN summary both backends must produce identically
  ;; ensure-schema WITH one IndexSchema -> put 3 rows carrying index-keys -> delete the middle
  ;; -> scan (base) -> scan-index (GSI) -> delete the SAME key again -> render a summary string
  )

(:wat::core::defn :user::compute [] -> :wat::core::String
  ;; start mem-store' and sqlite-store'(:memory:), run both, return the shared summary
  ;; IFF equal, else "DIFFERENTIAL-MISMATCH" — the sentinel the .rs asserts on.
  )
```

Return a **rendered summary**, not a bool: a bool that says "they agree" cannot tell you what
they agreed *on*, and two identically-broken backends would pass it.

## Blast radius

**`tests/rete/probe_arc301_delete_differential.wat` and `.rs` — NEW FILES ONLY.**
No production file is touched. `build.rs` auto-generates the module list from sibling `*.rs`
(`tests/rete/mod.rs` is an `include!` stub), so dropping the `.rs` in registers it — no
`mod.rs` edit.

If you find yourself editing anything under `wat/`, STOP — see STOP-1.

## STOP triggers

1. **If mem and sqlite disagree — STOP AND REPORT.** Do not edit `wat/query/sqlite-store.wat`
   or `wat/query/mem.wat` to reconcile them. Name which backend produced what, on which
   assertion. The disagreement is the deliverable.
2. **If a deleted row's GSI projection survives on either backend — STOP AND REPORT.** That is
   an orphaned-index bug and it invalidates stone 2's STOP-2 reasoning. Say so plainly.
3. **If `scan-index` cannot be driven from a test fixture** (a shape the template does not
   show) — STOP and report the gap rather than dropping the GSI half. Dropping it would make
   this stone vacuous.
4. **If the floor goes red on anything outside the two new test files** — STOP, capture the arm
   whole, do NOT re-run.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Read the Summary line. `cmd | tail` returns `tail`'s status; `grep -c` exits 1 on zero matches.
