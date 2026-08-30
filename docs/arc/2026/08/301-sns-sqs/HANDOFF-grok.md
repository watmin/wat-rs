# HANDOFF → grok — arc 301 stone 2: Store gains `delete`

Paste the block below to grok. Everything it needs is committed on `origin/sns-sqs`;
nothing here is a copy that can drift from the repo.

---

You are striking one stone in the `wat-rs` repo. Work on branch `sns-sqs`
(`git switch sns-sqs`, it tracks `origin/sns-sqs`).

**Read these two first, in full, before touching anything:**

- `docs/arc/2026/08/301-sns-sqs/BRIEF-stone-2-store-delete.md`
- `docs/arc/2026/08/301-sns-sqs/EXPECTATIONS-stone-2-store-delete.md`

The BRIEF names five rooms as exact `file:line`, tells you why you are being sent to each,
and carries an implementation sketch. Fill the sketch; do not invent the shape — each of the
three files already contains the `put` it is mirrored on.

**The gate.** `docs/arc/2026/08/301-sns-sqs/PROBE-store-has-no-delete.wat` is RED at HEAD on
exactly two errors (the second a cascade of the first). The stone is done when it is GREEN
**with no edit to the probe**. Everything else in that file is copied from the green
`tests/rete/probe_arc278_smem_roundtrip.wat` and type-checks clean today.

**Verify in the FOREGROUND and block on it.** Your turn ends when the numbers are in your
hands, not when a command is launched. Read the Summary line, never a piped exit code —
`cmd | tail` returns `tail`'s status and `grep -c` exits 1 on zero matches. This exact mistake
produced a "green" report on a 41-failure floor in this repo on 2026-08-30.

```bash
./target/release/wat --check docs/arc/2026/08/301-sns-sqs/PROBE-store-has-no-delete.wat
echo "CHECK=$?"
./scripts/floor.sh; echo "FLOOR=$?"
```

**On any red floor: do NOT re-run.** A green re-run destroys the only evidence. Capture the
failing arm whole — `scripts/floor.sh` already kept the untruncated log — name the exact
assertion, and report it. "Known flake" / "timing" / "pre-existing" / "unrelated to my change"
describe your search, never the failure.

**`.wat` is never hand-edited for a multi-site structural change** — that is a `wat-fix`
codemod (`wat/fix.wat`, recorded migrations in `wat-scripts/fixes/*.wat`). This stone is a
handful of single-site additions, so it should not need one; if you find yourself sweeping,
stop and say so.

**The four STOP triggers in the BRIEF are rejection criteria, not permission slots.** STOP-2
is the live one and it is the reason you may not improvise:

> `StoredRow` carries `index-keys` (its GSI projections). A `Key` carries only `(pk, sk)`.
> If deleting must ALSO remove index rows, the backend has to READ the row before deleting it —
> which breaks the symmetry with `put` this whole stone rests on. **If that fires, STOP and
> report it. Do not write a read-then-delete.** It is a design question and it belongs to the
> builder.

Same for a duplicate ack (deleting a key that is not present). Idempotent `:Success` is
probably right, but the brief does not state it and the probe does not test it — **if you have
to decide it, that is a finding to report, not a choice to make silently.**

**Report, when done:** each EXPECTATIONS scorecard row with its real result; the honest deltas
(what surprised you, what the brief got wrong); line counts per file; and anything you had to
decide that the brief did not cover.

---

## Grading (orchestrator, after grok reports)

Grade by **re-running**, never by reading the report. `examinare`: *no citation → phantom →
discarded.*

1. `git diff --stat` — blast radius is `wat/query.wat`, `wat/query/mem.wat`,
   `wat/query/sqlite-store.wat`, and nothing else.
2. `git diff -- docs/arc/2026/08/301-sns-sqs/PROBE-store-has-no-delete.wat` — **must be empty.**
   A moved gate is not a passed gate.
3. Re-run the probe and the floor yourself. Read the Summary line.
4. `./target/release/wat wat-scripts/demos/sns/sns-fanout.wat` — still `"3 3"`.
5. Read the diff for content integrity, not just green: did anything outside the targeted
   additions move?

Then write `SCORE-stone-2-store-delete.md` beside this file.

---

# HANDOFF → grok — stone 2b: the delete differential

Same branch, `sns-sqs`. Read in full before touching anything:

- `docs/arc/2026/08/301-sns-sqs/BRIEF-stone-2b-delete-differential.md`
- `docs/arc/2026/08/301-sns-sqs/EXPECTATIONS-stone-2b-delete-differential.md`

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
