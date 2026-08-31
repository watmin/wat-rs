# HANDOFF → grok — stone 2c: mem's `put` becomes a replace

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/NOTE-mem-store-put-appends-where-sqlite-replaces.md`
  — **including its ⛔ CORRECTED section.** The first version of that NOTE called this a tie
  between two defensible readings. It is not. Do not inherit that framing.
- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-2c-mem-put-is-a-replace.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-2c-mem-put-is-a-replace.md`

**This one is a bug fix with a settled referent.** `:wat::query::Store` is DynamoDB-shaped
(`wat/query.wat:7`); `PutItem` replaces; `sqlite-store` already does that and its DDL enforces
it. `mem-store` appends. Fix mem, say the rule on the surface, and cover re-put in the
differential so it cannot come back.

**Do not touch `wat/query/sqlite-store.wat`** — it is the reference (STOP-3).

The fixture already exists at `docs/excursus/2026/08/001-sns-sqs/PROBE-reput-divergence.wat`. Promote
it, drop its standalone `:user::main`, add a `.rs` harness. At HEAD it prints
`MEM[base=2:a,a;gsi=2:v1,v9]  SQLITE[base=1:a;gsi=1:v9]`; after the fix both must read
`base=1:a;gsi=1:v9`.

STOP-1 is the live one: **if fixing mem reds an existing test, capture the arm and report it.**
That test was asserting a state DynamoDB cannot represent, so the fix is probably the test —
but which test it is, is the finding.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. On a red floor: do
NOT re-run. Floor at HEAD is 5096.
