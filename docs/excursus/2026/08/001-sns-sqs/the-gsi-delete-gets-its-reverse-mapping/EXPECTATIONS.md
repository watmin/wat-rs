# EXPECTATIONS — the GSI delete gets its reverse mapping

Written **before** the strike. `wat/query/sqlite-store.wat` (`ensure-schema` only) plus one probe.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★★ **no orphan GSI rows** | a new probe | after put→delete cycles, every `index_*` table has **zero** rows whose `(pk, sk)` is absent from `main`. Checked directly, not inferred |
| 2 | ★★ **`count-index` still reports the true depth** | the same probe | depth after a delete cycle equals the live row count. An orphan inflates depth, which drives `room = cap - depth` and therefore **admission** |
| 3 | ★ **`delete`'s per-call latency flattens** | the three-point sweep | `4000/1000` drops from **1.966** toward 1.0, with bands |
| 4 | ★ **`put`'s cost is measured, not assumed** | same | `put`'s `4000/1000` reported against its **1.514** baseline. A second b-tree per GSI row is real work |
| 5 | ★ **the NET effect on `drain` is stated** | same | `drain` median and `4000/1000` against the `ba63bedf3` tree. **If the put cost exceeds the delete win, say so and recommend against landing** |
| 6 | **the read path is unmoved** | same | `count-index` ≈1.27, `scan-index` ≈0.89 — an index on `(pk, sk)` must not perturb `(ipk, isk)` queries |
| 7 | **correctness at every point** | every run | `distinct = n×m`, `dup=0` |
| 8 | **blast radius** | `git diff --stat` | `wat/query/sqlite-store.wat` only, plus the probe. **No `mem.wat`. No `src/`. No `sqs.wat`** |
| 9 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 10 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★★ **Rows 1 and 2 outrank every timing row.** This touches the delete path of the store that backs
admission. An orphan GSI row inflates `depth`, `room = cap - depth` shrinks, and the queue starts refusing
publishers for capacity it actually has — the exact failure this arc spent hours characterising. **A
faster delete that leaves orphans is strictly worse than a slow correct one.**

★ **Row 5 is the stone's actual verdict, and it may be "do not land".** An added index makes every `put`
maintain a second b-tree. `put` already grows 1.514×. If that cost exceeds the delete win, **the honest
outcome is a recommendation against the change**, with the numbers. That is a complete result.

⚠ **Row 6 is the regression guard.** The `(pk, sk)` index must not change the query plan for
`ipk=? AND isk BETWEEN ?`. If `count-index` or `scan-index` moves materially, SQLite has chosen a
different index and that needs naming.

## Runtime prediction

**60–90 minutes.** One `CREATE INDEX` per GSI table, one probe, then ~7 minutes of sweep and ~8 of floor.
The probe is the substance; the SQL is one line.

## Trap-doors

- **`IF NOT EXISTS`** — `ensure-schema` is called on every store start and must stay idempotent.
- **The index name must not collide** across GSI tables. `index_{name}_by_key` derives from the GSI name.
- **`ensure-schema` runs inside `setup`**, which is CONSTANT (~12.4 s) and out of scope. Creating an index
  on an empty table is free; do not let this stone drift into setup work.
- **An orphan is invisible to `distinct`/`dup`.** Those count delivered messages, not index rows. Row 1
  needs its own query.
- **Do not touch `mem.wat`** — same contract, different cause, out of scope.
- **`put` writes main AND every GSI table.** When reading its latency, remember it was already doing two
  b-trees per GSI row; this adds a third.

## What this stone does NOT claim

⚠ It makes **no prediction** about the net effect. Ten mechanism claims of mine have been refuted in this
arc; row 5 exists so the numbers decide.
⚠ It does **not** address `put`'s 1.514× or `count-index`'s 1.268× — both predicate correctly and are a
separate story.
⚠ It does **not** touch `mem.wat`, `sqs.wat`, or the counter-carrier tax.
