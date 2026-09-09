# DESIGN — the GSI delete gets its reverse mapping

**A failed replication, measured then named.** `sqlite-store.wat` declares itself DDB-faithful; its GSI
delete path is not.

## The defect

`wat/query/sqlite-store.wat:18` — *"DDB-faithful — a GSI is a SEPARATE table"*. `:22` — *"DELETE this
row's old projection from EVERY declared GSI table"*.

```sql
:211   CREATE TABLE index_{name} (ipk, isk, pk, sk, data, PRIMARY KEY(ipk, isk, pk, sk))
:228   DELETE FROM [index_{name}] WHERE pk=? AND sk=?
```

**The primary key leads with `(ipk, isk)`; the delete predicates on `(pk, sk)` — the trailing segments.**
A b-tree on `(ipk, isk, pk, sk)` cannot serve a lookup by `pk, sk`, so **every GSI-row delete is a full
table scan.**

## ★★ Why this is a replication failure and not a tuning oversight

DynamoDB's contract is **delete by base key**. A caller never addresses a GSI entry directly; DDB
propagates the removal. The emulation replicated that interface exactly — and `:300-301` shows the choice
was deliberate and reasoned:

> *"DELETEs each `index_<name>` by (pk, sk) — the columns those tables already carry. No read of the row,
> no index-keys on Key."*

Avoiding a read to locate the projection is precisely what DDB does. **DDB can, because it maintains an
internal mapping from base key to GSI entries. The emulation has no such mapping.** The interface was
replicated; the structure that makes it cheap was not.

★★★ So the corrective change is not "add an index for speed" — it is **completing the replication**: give
each GSI table the base-key mapping DDB keeps internally, so delete-by-base-key is a seek here as it is
there.

## The measurement that found it

`ba63bedf3`, per-call latency, four subscriber tiers, medians with bands:

| op | 4000/1000 | predicate | index-covered? |
|---|---|---|---|
| **`delete`** | **1.966** | `pk=? AND sk=?` | **NO — trailing key segments** |
| `put` | 1.514 | insert | (writes both tables) |
| `count-index` | 1.268 | `ipk=? AND isk BETWEEN ?` | yes — leading segments |
| `scan-index` | 0.890 | `ipk=? AND isk BETWEEN ?` | yes — **flat** |

★ And the sweep's own control: the **inbox is capped at `:cap 64`** live rows and is **flat on all four
ops** — same code, same store. Growth tracks **table size**, which is exactly what a scan predicts and a
seek does not.

## The change

One statement per GSI table in `ensure-schema` (`:200-211`):

```sql
CREATE INDEX IF NOT EXISTS [index_{name}_by_key] ON [index_{name}] (pk, sk)
```

Additive. No interface change, no semantic change, no change to what rows a delete removes.

## ⛔ The cost this must measure, not assume

An added index means **every `put` maintains a second b-tree per GSI row.** `put` already grows 1.514×.

**So the stone is not "delete gets faster" — it is "the net effect on `drain` is measured."** If the put
cost exceeds the delete win, that is the finding and the index should not land. Row 3 gates this; no
prediction is offered.

## ⛔ And the correctness risk that matters more than either

If the delete's row-matching changes at all, **orphan GSI rows** appear — and `count-index` derives
`depth`, which drives `room = cap - depth` and therefore **admission**. An orphan would silently inflate
depth and start refusing publishers.

Row 5 requires a direct probe that no orphans remain after a delete cycle, not an inference from
`distinct`/`dup`.

## `mem.wat` shares the contract, not the cause

`wat/query/mem.wat:96-107` — `key-hits-row?` compares `(pk, sk)` per row, folded by
`row-in-delete-batch?`. Same DDB contract, but it has **no index structure at all**, so O(rows) is its
nature rather than a missed mapping. **Named; not in this stone.** The drain uses `sqlite-store`
(`circuit.wat:2142`, `:2160`).

## OUT OF SCOPE — REJECTED

- **`mem.wat`.** Different cause, different fix, and not on the measured path.
- **`put`'s 1.514× and `count-index`'s 1.268×.** Both predicate correctly, so their growth is a separate
  story — plausibly b-tree depth or the per-call transaction. Named, not chased here.
- **The counter-carrier tax** (~10 % of drain from `ba63bedf3`). Independent, and a constant multiplier
  does not distort this stone's ratios.

## Files

`wat/query/sqlite-store.wat` — `ensure-schema` only — plus a probe under `wat-scripts/scratch-pad/`.
