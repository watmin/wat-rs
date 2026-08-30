# SCORE — arc 301 stone 2: Store gains `delete`

**STRUCK.** Graded 2026-08-30 against the orchestrator's OWN re-run, never the executor's
report. Executor: grok (named per `examinare` — not a spawned rider, not Opus).

## The scorecard, re-run

| # | what | expected | **measured by me** |
|---|---|---|---|
| 1 | probe GREEN, unedited | `CHECK=0` | ✅ `CHECK=0` |
| 2 | assertion actually runs | pass, not skipped | ✅ `PASS [0.716s] (456/5095) wat::rete probe_arc301_store_delete::store_delete_removes_exactly_the_named_row` |
| 3 | probe not edited | empty diff | ✅ **0 bytes** |
| 4 | blast radius | 3 files | ⚠️ 3 production files + 2 promoted test files — see *The contradiction was mine* |
| 5 | floor | `FLOOR=0` | ✅ `Summary [ 294.592s] 5095 tests run: 5095 passed (2 slow), 17 skipped` |
| 6 | journal both backends | green | ✅ `PASS [0.845s] journal_persists_identically_across_mem_and_sqlite_backends_on_a_thread` |
| 7 | SNS demo unaffected | `"3 3"` | ✅ `"3 3"`, exit 0 |
| 8 | mem green, sqlite deferred | as specified | ✅ mem only |

**Test-count check the report did not claim:** this branch's floor was **5094** before the
stone and **5095** after. Exactly +1 — the new test, and nothing silently disabled or ignored.

**Build check:** `cargo build --release --tests --workspace` finished in **0.10s** — a no-op,
so I graded the same artifact grok built, not a rebuild of my own.

## STOP-2 did not fire — and I verified the reasoning, not the claim

grok reported `Key` sufficient. Checked independently against HEAD:

- `clear-index-projections` **pre-exists** (`wat/query/sqlite-store.wat:155` at HEAD, not added
  by this stone). It takes `(conn, names, pk, sk)` and issues
  `DELETE FROM [index_{name}] WHERE pk=? AND sk=?`. **No read of the row.**
- mem's `:durable` is a single `rows <- (PersistentVector :- [StoredRow])` — **no separate
  index structure at all**, so dropping a row necessarily drops its projection.

Both halves hold. And one point stronger than grok claimed: **`put-one-row` already issues the
identical `DELETE FROM main WHERE pk=? AND sk=?`** (`:207`, its clear-then-insert), so
`delete-one-key` reuses a statement shape already proven in production.

## Content integrity — read, not inferred from green

All three diffs are **pure insertion**: 23 / 37 / 54, zero deletions, `put`/`scan`/`scan-index`
lines unmoved. `DeleteResponse` is `PutResponse`'s arm list verbatim. `delete-response` was
**minted beside** `put-response`, not widened into it — trap-door 2 handled exactly as briefed.
`delete-rows` mirrors `put-rows`'s recursion; the impl is the same `begin → … → commit` chain.

Registration is correct with no `mod.rs` edit: `tests/rete/mod.rs` is an `include!` stub and
`build.rs` auto-generates the module list from sibling `*.rs` — *"Add a test: drop a .rs here."*

## The contradiction was MINE, not the executor's

Row 4 is the only non-clean row, and the cause is that my own two artifacts disagreed:

- **BRIEF** said *"When the stone lands, promote it — into `tests/rete/` … so it becomes a
  permanent floor test rather than an arc artifact."*
- **EXPECTATIONS row 4** said the blast radius is *"exactly `wat/query.wat`,
  `wat/query/mem.wat`, `wat/query/sqlite-store.wat` (+ the SCORE)."*

grok followed the BRIEF, promoted the probe, **and reported the conflict rather than quietly
picking one.** That is correct executor behaviour against a defective brief. The promoted copy
is `cmp`-clean against the gate. **Lesson: an EXPECTATIONS row that enumerates files must be
derived from the BRIEF's own instructions, not written beside them.**

## Findings — carried forward, none blocking

1. **The sqlite `delete` path has ZERO test coverage.** 54 of the 114 new lines are exercised
   by nothing. Row 8 specified mem-only so this is not a violation, but **stone 2b (the
   mem-vs-sqlite differential) is now load-bearing, not optional.** The static evidence above
   is strong; it is not a test.
2. **Duplicate ack / missing key is untested.** grok reports `:Success` and — correctly — did
   **not** invent a `NotFound` arm, on the grounds that `DeleteResponse` is `PutResponse`'s arm
   list and SQL `DELETE` of 0 rows is `Ok`. That is the arm list talking, which is the right
   reasoning. But SQS acks are at-least-once, so **a duplicate ack is normal traffic and must
   become a probe row** (EXPECTATIONS trap-door 3, now a finding).
3. **mem's `delete` is O(rows × keys).** `row-in-delete-batch?` folds every key for every row.
   mem-store is a test backend so this is not a defect here — but a queue drains continuously,
   and **stone 3 must not build on this unexamined.**

## Executor assessment

Clean strike. The gate held untouched; the STOP trigger was *checked and reported as
not-firing with its reasoning*, rather than assumed; trap-door 2 was resolved the briefed way;
and the two decisions the brief did not make (missing-key semantics, `delete-response` vs
widening) were **surfaced as findings rather than made silently** — which is the exact failure
mode the handoff was written to prevent, and it did not occur.
