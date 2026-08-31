# HANDOFF → grok — excursus 001 stone JOURNAL-CENSUS

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-journal-census.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-journal-census.md`

**This stone changes NO production code and fixes NOTHING. It measures.** The deliverable is a
table. A RED table is the success case.

`journal`'s base key is `(namespace + kind, time-ns)` — nothing in it says *which* metric. One
fixture (`span_macros`) is known to get 3 rows on mem and **1** on sqlite. **Fourteen more have
never run against the real backend.** The `SortKey` fix changes a stored key layout, and doing
that on one fixture's worth of evidence is guessing. Get the number first.

**The exemplar is `tests/services/probe_arc278_journal_backend_differential.wat`** — it already
swaps the backend for one path: a helper parameterized on the store's Address, both stores
started, run both, compare. `:index-names` must carry `"by-uuid"` because journal's `:init`
declares that GSI.

★ **STOP-1 is the control:** `span_macros` MUST show the disagreement (mem 3, sqlite 1). If your
instrument says it agrees, the instrument is not swapping the backend and every other row is
worthless. Report that rather than the table.

⚠ **Trap-door 1 matters as much as the reds:** a fixture can agree and still be lying —
agreement only covers the sequence it drove. `journal_backend_differential` agreed for months
while `span_macros` lost 2 of 3 metrics, because it never wrote two metrics at one nanosecond.
**When a fixture agrees, note what it actually exercised**, not just that it agreed.

Do NOT edit fixtures in place — the floor's one known red is load-bearing evidence. Work on
copies and delete them before the commit. Floor must be unchanged: **5119, one known failure,
before and after.**
