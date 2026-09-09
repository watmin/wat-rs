# BRIEF — the GSI delete gets its reverse mapping

## The work, in one paragraph

`sqlite-store.wat` models DynamoDB: a GSI is a separate table, and a delete removes the row's projection
from every GSI **by base key**. That interface is DDB-faithful. But each GSI table's only index is
`PRIMARY KEY(ipk, isk, pk, sk)`, and the delete predicates on `(pk, sk)` — the *trailing* segments — so
every GSI-row delete is a full table scan. DDB can do this cheaply because it keeps an internal base-key →
GSI mapping; the emulation replicated the interface without the structure. Add that mapping as an index,
then **measure whether the net effect is a win** — because it also makes every `put` maintain another
b-tree.

## Read in order

1. **`wat/query/sqlite-store.wat:18-23`** — the ratified design's own words: *"DDB-faithful — a GSI is a
   SEPARATE table"* and *"DELETE this row's old projection from EVERY declared GSI table"*. This is the
   contract you are completing, not changing.
2. **`wat/query/sqlite-store.wat:200-211`** — `ensure-schema`, and the `CREATE TABLE` whose
   `PRIMARY KEY(ipk, isk, pk, sk)` is the whole problem. **Your `CREATE INDEX` goes here.**
3. **`wat/query/sqlite-store.wat:228`** — `DELETE FROM [index_{name}] WHERE pk=? AND sk=?`, the scan.
4. **`wat/query/sqlite-store.wat:299-301`** — why it was written this way: *"No read of the row, no
   index-keys on Key."* Deliberate and reasoned; it just lacked the mapping.
5. **`docs/excursus/2026/08/001-sns-sqs/the-store-reports-time-per-operation/SCORE.md`** — the per-op
   baseline you are measuring against, and its medians-with-spread discipline.

## Implementation sketch

```wat
;; in ensure-schema, once per declared GSI name, beside the CREATE TABLE:
(:wat::core::format
  "CREATE INDEX IF NOT EXISTS [index_{name}_by_key] ON [index_{name}] (pk, sk)"
  :name name)
```

Then the probe (`wat-scripts/scratch-pad/`): put rows into a store with a declared GSI, delete a subset by
base key, and assert **zero** `index_*` rows whose `(pk, sk)` is absent from `main`, and that
`count-index` reports exactly the live count.

Then the sweep: `circuit.wat <n> 4 3 8192 true 1000` for n ∈ {1000, 2000, 4000}, ≥3 runs each, quiet box,
reporting per-op `ns/calls` and `drain`, against the `ba63bedf3` baseline.

## Blast radius

`wat/query/sqlite-store.wat` — `ensure-schema` only — plus one probe under `wat-scripts/scratch-pad/`.
**No `mem.wat`. No `src/`. No `sqs.wat`. No `circuit.wat`.**

## STOP triggers

**STOP-1** — if **any** orphan GSI row can exist after a delete cycle, **STOP**. `count-index` derives
`depth`, `depth` drives `room = cap - depth`, and that drives admission: an orphan makes the queue refuse
publishers for capacity it has. A faster delete that leaks index rows is strictly worse than a slow
correct one.

**STOP-2** — if `put`'s per-call latency growth exceeds the `delete` win, **STOP and recommend against
landing**, with the numbers. That is a complete and valuable result, not a failure. Do not land a change
that is net-negative because the stone was drawn expecting a win.

**STOP-3** — if `count-index` or `scan-index` latency moves materially, **STOP and name it.** SQLite has
chosen a different query plan, and that is a finding about the schema.

**STOP-4** — do **not** touch `mem.wat`, `sqs.wat`, `circuit.wat`, or anything under `src/`. One file and
one probe.

**STOP-5** — do **not** change the `DELETE` statement, the delete semantics, or what rows a delete
removes. The change is additive: a new index, nothing else.

**STOP-6** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

A probe proving **zero orphan GSI rows** and a truthful `count-index` after delete cycles — that is the
gate, and it outranks every timing number. Then a sweep table with per-op `ns/calls` at n=1000/2000/4000,
`delete`'s `4000/1000` against its **1.966** baseline, `put`'s against **1.514**, `count-index` against
**1.268**, `scan-index` against **0.890**, and `drain`'s median and ratio against the `ba63bedf3` tree.
`distinct = n×m`, `dup=0` everywhere. Load stated per run. `every_wat_scripts_file_loads` PASS. Floor
Summary read after the sweep: 5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

**Write the SCORE to
`SCORE.md`** in the shape of the
neighbouring SCORE files. **Do not commit.**

State plainly whether the net effect is a win, a wash, or a loss — **and recommend accordingly.** No
prediction was made; the numbers decide.
