# EXPECTATIONS — excursus 001 stone 2: Store gains `delete`

**Written BEFORE the strike, 2026-08-30, so the result cannot move the goalposts.**
Every row is a command and an outcome. The SCORE is written against the orchestrator's OWN
re-run of these, never against the executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe goes GREEN, unedited | `./target/release/wat --check docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat; echo "CHECK=$?"` | `CHECK=0` |
| 2 | the probe's assertion actually runs | the deftest executes; scan count goes 3 → 2 | pass, not skipped |
| 3 | the probe was NOT edited | `git diff --stat -- docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat` | **empty** |
| 4 | blast radius held | `git diff --name-only` | exactly `wat/query.wat`, `wat/query/mem.wat`, `wat/query/sqlite-store.wat` (+ the SCORE) |
| 5 | the floor | `./scripts/floor.sh; echo "FLOOR=$?"` | `FLOOR=0`, Summary line read verbatim |
| 6 | journal still works on BOTH backends | the existing `probe_arc278_journal_backend_differential` arm | green — it is the regression canary |
| 7 | SNS demo unaffected | `./target/release/wat wat-scripts/topic/sns-fanout.wat` | `"3 3"` |
| 8 | both backends AGREE | the probe runs against `mem-store`; a sqlite-backed twin is stone 2b | mem green; sqlite deferred, see below |

## Runtime prediction

**45–90 minutes.** Three files, all short and symmetric; the shape is copied from `put` in
each. The type/record additions are ~20 lines of `wat/query.wat`; `mem.wat`'s impl is an
inverted fold (~12 lines); `sqlite-store.wat` needs `delete-rows` beside `put-rows` (~14 lines)
plus the `begin → … → commit` chain (~10). A full `cargo build --release --tests` is ~1m20s
here and the floor is ~5m20s, so the verify tail alone is ~7 minutes.

## Trap-doors — named now, so a SCORE cannot discover them as excuses

1. **`Key` may not be sufficient.** `StoredRow` carries `index-keys` (its GSI projections); a
   `Key` carries only `(pk, sk)`. If deleting must also remove index rows, the backend may need
   to READ the row before deleting it — which makes `delete` non-symmetric with `put` in a way
   this brief did not anticipate. **That is STOP-2, and it is the builder's call.** If it fires,
   the honest outcome is a reported gap, not an improvised read-then-delete.
2. **`put-response` may not admit `DeleteResponse`.** `wat/query/sqlite-store.wat:51` maps a
   chained `Result` onto `PutResponse`. If its return type is concrete, a `delete-response`
   twin is needed. Minting the twin is fine; *widening* `put-response` to be generic is a
   larger change than this stone and would need saying.
3. **Deleting a key that is not present.** SQS acks are at-least-once, so a duplicate ack is
   normal traffic. Idempotent `:Success` is almost certainly right, but the brief does not
   state it and the probe does not test it. **If the implementer has to decide this, it is a
   finding to report, not a choice to make silently** — and it should become a probe row.
4. **A green floor here means MAIN is green.** This branch is cut from `origin/main`, which
   moved twice today. If the floor reds on something in `src/`, check whether it reds on
   `origin/main` too before attributing it to this stone — but capture the arm FIRST, whole,
   and do not re-run.

## What is explicitly NOT in this stone

- **The sqlite differential.** Row 8 above runs `delete` against `mem-store` only. A
  sqlite-backed twin of the probe — the mem-vs-sqlite differential this arc's design leans on —
  is **stone 2b**, and pretending it landed here would be the arc's first lie.
- **The queue itself.** `wat/queue.wat` is stone 3. This stone only makes `ack` expressible.
- **`:peers` seeing through containers.** Named in DESIGN, deliberately not drawn.
