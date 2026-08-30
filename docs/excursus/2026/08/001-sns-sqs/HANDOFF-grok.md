# HANDOFF → grok — excursus 001 stone 2: Store gains `delete`

Paste the block below to grok. Everything it needs is committed on `origin/sns-sqs`;
nothing here is a copy that can drift from the repo.

---

You are striking one stone in the `wat-rs` repo. Work on branch `sns-sqs`
(`git switch sns-sqs`, it tracks `origin/sns-sqs`).

**Read these two first, in full, before touching anything:**

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-2-store-delete.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-2-store-delete.md`

The BRIEF names five rooms as exact `file:line`, tells you why you are being sent to each,
and carries an implementation sketch. Fill the sketch; do not invent the shape — each of the
three files already contains the `put` it is mirrored on.

**The gate.** `docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat` is RED at HEAD on
exactly two errors (the second a cascade of the first). The stone is done when it is GREEN
**with no edit to the probe**. Everything else in that file is copied from the green
`tests/rete/probe_arc278_smem_roundtrip.wat` and type-checks clean today.

**Verify in the FOREGROUND and block on it.** Your turn ends when the numbers are in your
hands, not when a command is launched. Read the Summary line, never a piped exit code —
`cmd | tail` returns `tail`'s status and `grep -c` exits 1 on zero matches. This exact mistake
produced a "green" report on a 41-failure floor in this repo on 2026-08-30.

```bash
./target/release/wat --check docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat
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
2. `git diff -- docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat` — **must be empty.**
   A moved gate is not a passed gate.
3. Re-run the probe and the floor yourself. Read the Summary line.
4. `./target/release/wat wat-scripts/demos/sns/sns-fanout.wat` — still `"3 3"`.
5. Read the diff for content integrity, not just green: did anything outside the targeted
   additions move?

Then write `SCORE-stone-2-store-delete.md` beside this file.

---

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

---

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

---

# HANDOFF → grok — stone INST: `#inst` renders at constant nanosecond width

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-inst-renders-at-constant-width.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-inst-renders-at-constant-width.md`

**One token in the substrate.** `crates/wat-edn/src/writer.rs:227` uses
`SecondsFormat::AutoSi` — chrono's "shortest representation that is a multiple of 3 digits" —
so `1.200000000s` prints `.200Z` and `1.200000100s` prints `.200000100Z`. `'Z'` (0x5A) sorts
after `'0'` (0x30), so **the earlier instant compares greater**, and every range `scan` over a
timestamp sort key is unsound. `SecondsFormat::Nanos` always emits 9.

`crates/wat-edn/src/json.rs:170` has the same call. **That one is a DECISION, not a
copy-paste** — EDN's `#inst` is a sort key in this system, JSON's is an interchange value.
Either answer is fine; an unstated one is not. Say what you did in the SCORE.

**The gate** is `PROBE-inst-lexicographic-order-is-not-chronological.wat`, already committed.
At HEAD its comparisons give
`9-digit=false whole-second=false 6-digit=false 3-digit=false control=true widths=32/38/28`.
Done when all six deftests pass **with no edit to the probe**, and it is promoted into
`wat-tests/` so the property stays on the floor.

⚠ **THE FLOOR IS ALREADY RED AND THAT RED IS NOT YOURS.**
`probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close` fails at
HEAD — the journal key-collision bug stone 2c exposed. **Expected: exactly ONE failure, that
one.** Two means you added one. Do not fix the span arm; it is drawn separately.

Measured golden churn is **zero** — that is a claim in the BRIEF, and STOP-1 says if a golden
does churn, report it, because a wrong census matters more than the golden.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. On a NEW red: do NOT
re-run, capture the arm whole, name the exact assertion.

---

# HANDOFF → grok — excursus 001 stone WRITE-OPTS

⚠ **This work moved.** It was `docs/arc/2026/08/301-sns-sqs/` and is now
`docs/excursus/2026/08/001-sns-sqs/`. It is NOT an arc — arc 301 does not exist; the number was
minted unasked and retracted. Commit prefix is `EXCURSUS(001):`, never `STONE n(NNN):`. The six
`probe_arc301_*` tests are now `probe_ex001_*`. See `docs/excursus/README.md`.

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-write-opts.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-write-opts.md`

**The builder rejected three designs before this one.** Do not re-propose them:
a global config knob (a footgun — one setting and every `StoredRow` written afterwards loses
its range-scan ordering); a fixed default in `json.rs` (frozen from an assumption about a
consumer nobody asked); and a bare `digits` parameter (a timestamp concern on a general
serializer's signature — the wrong axis).

**What ships: a `WriteOpts` VALUE the caller passes**, on the `ProcessOpts` precedent already in
the tree at `wat/spawn.wat:77/122/130` — a struct, a zero-arg default constructor you never
touch, and named single-field variants. This excursus's own SNS demo uses both halves of that
pattern already.

⛔ **`:wat::edn::write` (the 1-arg EDN verb) does not change.** 424 call sites, and it is the
`Store` sort-key path — its width is a correctness invariant, not a preference. If opts cannot
be added to the JSON verbs without touching it, that is a finding, not a licence.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. Floor here is **5103
with ONE known failure** (`probe_arc278_span_macros`, the journal key-collision arm). **That red
is expected and is not yours.** Two failures means you added one. On a NEW red: do NOT re-run,
capture the arm whole, name the exact assertion.

---

# HANDOFF → grok — excursus 001 stone WO-OPT: the opts arg becomes OPTIONAL

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-write-opts-optional.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-write-opts-optional.md`

**This corrects MY brief, not your work.** WRITE-OPTS shipped `write-json` at a required arity
of 2 because my sketch showed `(:wat::edn::write-json v (:wat::edn::opts))` everywhere. The
builder's intent was: *"if you omit it, you get the defaults; if you want to change it, you pass
the config ops you want for your call."* You built what I wrote; the specification was wrong.

`(:wat::edn::write-json v)` must type-check and mean the default.

**The exemplar is exact — `:wat::io::IOReader/read-frame`**, which already accepts 1 or 2 args:
a Variadic handler (`src/intrinsic/io/reader.rs:410`), the arity guard in a named `infer_` fn in
the checker (`src/check.rs:9281`), and a dispatch arm that intercepts it (`src/check.rs:2977`).
`reader.rs:80` calls it out as the one exception of ten — do the same for the JSON verbs.

⚠ **Do NOT add a `Range` arity to the intrinsic registry.** `src/intrinsic/mod.rs:142` says
Range/AtLeast are deliberately out of scope. That is a registry-shape change and it belongs to
arc 255. If Variadic-plus-checker-guard is unworkable, STOP and report.

★ **Row 2 is the real gate:** `(write-json v)` and `(write-json v (:wat::edn::opts))` must
produce **byte-identical** output. "1-arg works" would pass with any default.

`:wat::edn::write` / `write-pretty` stay `Exact(1)` — sort-key path, unchanged. `wat/edn.wat`
and `crates/wat-edn/` are untouched.

`.contains(` on a deterministic string trips `no_loose_string_assert` — it has caught two stones
in this excursus already. Use `assert_eq!` on the whole string from the start.

Verify in the FOREGROUND; read the Summary line. Floor is **5113 with ONE known failure**
(the journal key-collision arm) — expected, not yours.

---

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

---

# HANDOFF → grok — excursus 001 stone SORTKEY

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-sortkey.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-sortkey.md`
- `docs/excursus/2026/08/001-sns-sqs/NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`
  — **including its ⛔ CORRECTED section.** The table at its top is half a measurement; the bug
  is NOT sqlite-specific.

**This is the excursus's largest stone, and the first expected to produce a FULLY GREEN floor.**
Every prior stone ran against a known red. This one fixes it. `FLOOR=0` is the target.

`journal`'s base key is `(namespace + kind, time-ns)` — nothing in it says WHICH event. A span's
close emits three Metrics at one instant, they share a key, `put` replaces, two are lost.
Measured on **both** backends: `span_macros` gets 1 where it asserts 3.

**The ruling is option C** (the four questions, all four YES): a telemetry event carries its own
id. `Scope` is a `defsurface` spliced into `Metric` and `Log`, so **one field reaches both**.
The user-facing surface — `(:wat::telemetry::log span :Info "…")`, `incr`, `timed` — does not
change at all; users never construct these records.

★ **Two things will decide whether this works:**

**Row 4** — the three Metrics of one `close` share `now`. That sharing IS the bug. If they end
up sharing an event id too, nothing is fixed and `span_macros` still returns 1.

**Row 7 / STOP-2** — the range bounds. A `SortKey` record renders longer than a bare timestamp,
so a row at exactly `time-hi` must still fall inside `sk-hi`. If the maximal sentinel is not
truly maximal, `query-metrics` silently drops the newest data **and every existing fixture
still passes**, because none queries a boundary. **Demonstrate it; do not argue it.** Same
class as the `#inst` width bug — an ordering property nothing asserted.

The BRIEF's census is **known-incomplete and says so**. Re-derive it and report your number.

Verify in the FOREGROUND; read the Summary line. On a NEW red: do NOT re-run, capture the arm
whole, name the exact assertion.
