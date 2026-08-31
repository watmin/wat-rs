# HANDOFF → grok — stone 2b: the delete differential

Same branch, `sns-sqs`. Read in full before touching anything:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-2b-delete-differential.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-2b-delete-differential.md`

**This stone is different from every other one here.** It adds NO production code, and its
outcome is genuinely unknown — the sqlite `delete` path has never executed. **If mem and
sqlite disagree, the disagreement IS the deliverable.** Report it; do not edit either backend
to make the numbers match. A differential that gets quietly reconciled has destroyed the only
evidence it existed to produce.

**The GSI is the point, not a detail.** Stone 2's STOP-2 argued that a `(pk, sk)` `Key` is
sufficient *because* `clear-index-projections` deletes index rows by those columns. That
argument is entirely about the GSI path. An empty `:index-names` makes
`clear-index-projections` return `Ok` immediately (`wat/query/sqlite-store.wat:155`), so a
fixture without an index would go green and prove nothing. Declare one index, have rows
project into it, and drive `scan-index` AFTER the delete on both backends.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. On a red floor: do
NOT re-run, capture the arm whole, name the exact assertion.

Report each EXPECTATIONS row with its real result, plus anything you had to decide that the
brief did not cover.
