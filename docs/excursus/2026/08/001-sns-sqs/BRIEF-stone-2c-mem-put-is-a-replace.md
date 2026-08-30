# BRIEF — excursus 001 stone 2c: `mem-store`'s `put` becomes a replace-by-`(pk, sk)`

**Builder's ruling 2026-08-30: `mem-store` is the bug; fix it.** Not a contract question —
see `NOTE-mem-store-put-appends-where-sqlite-replaces.md`, including its ⛔ CORRECTED section.

## The work, in one paragraph

`:wat::query::Store` is explicitly DynamoDB-shaped (`wat/query.wat:7`). **`PutItem` replaces**:
an item whose primary key already exists is completely replaced, and a DDB table cannot hold
two items with one primary key. `sqlite-store` implements this correctly (`put-one-row` is
`DELETE → clear-index-projections → INSERT → insert-index-projections`) and its DDL enforces it
(`PRIMARY KEY(pk,sk)`). `mem-store`'s `put` is a bare `conj` — it appends, producing two rows
with one key and leaving both GSI projections alive. Make mem's `put` a replace, say the rule
on the surface, and extend the differential so the class cannot return.

## Read in order

1. **`NOTE-mem-store-put-appends-where-sqlite-replaces.md`** — the measurement and the
   correction. Read the ⛔ CORRECTED section; the first version of that NOTE was wrong and you
   should not inherit its framing.
2. **`wat/query/mem.wat`, the `put` impl** — `foldl … (:wat::vector::conj acc r)` over
   `Record/rows`. This is the defect, and the fix is local to it.
3. **`wat/query/mem.wat`, `:wat::query::key-hits-row?` and `:wat::query::row-in-delete-batch?`**
   — stone 2 already wrote the `(pk,sk)` predicate you need. **Reuse them; do not write a
   second one.** A replace is "drop any existing row whose `(pk,sk)` an incoming row names,
   then append the batch."
4. **`wat/query/sqlite-store.wat`, `put-one-row`** — the reference semantics. Note it clears
   the GSI projections *before* re-inserting; mem derives projections from rows, so dropping
   the old row is what corresponds.
5. **`wat/query.wat:551–569`** — `put`'s `:features` doc-comment, currently *"write a batch
   ATOMICALLY (one transaction)"*. It says nothing about keys, which is how this was misread
   once already.

## Three parts, all required

**(a) `mem-store`'s `put` replaces.** Existing rows whose `(pk,sk)` appears in the incoming
batch are dropped; then the batch is appended. Within a batch, later rows win over earlier ones
naming the same key — that is what a sequential `PutItem` stream does, and what sqlite's
`put-rows` recursion already produces.

**(b) Say the rule on the surface.** Extend `put`'s doc-comment in `wat/query.wat` to state
replace-by-`(pk,sk)` semantics explicitly. Right now the governing model is a prose aside on
line 7 of that file; **a contract whose key semantics live in another document's header is a
contract that will be misread again** — it already was, by me. Also correct
`docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md:150`, whose backend table maps `put`
to a plain `INSERT` (which against `PRIMARY KEY(pk,sk)` would error, not replace).

**(c) The differential covers re-put.** `docs/excursus/2026/08/001-sns-sqs/PROBE-reput-divergence.wat`
is the fixture, already committed. Promote it into `tests/rete/` beside
`probe_ex001_delete_differential`, **drop its standalone `:user::main`** (the harness drives
`:user::compute`), and add a `.rs` harness pinning the agreed summary. At HEAD it produces:

```
MEM[base=2:a,a;gsi=2:v1,v9]     SQLITE[base=1:a;gsi=1:v9]
```

After the fix both sides must read `base=1:a;gsi=1:v9`.

## Blast radius, derived from (a)/(b)/(c) above

- `wat/query/mem.wat` — the `put` impl
- `wat/query.wat` — `put`'s doc-comment only, **no signature or type change**
- `docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md` — the one table row
- `tests/rete/probe_ex001_reput_differential.{wat,rs}` — new
- this arc's SCORE

**`wat/query/sqlite-store.wat` is NOT touched. It is already correct.**

## STOP triggers

1. **If fixing mem's `put` reds any existing test** — STOP, capture the arm whole, do NOT
   re-run, and report it. A test that depended on append semantics was asserting a state
   DynamoDB cannot represent, and *which* test that is, is the finding.
2. **If a batch containing two rows with the same `(pk,sk)` has no obvious winner** — the rule
   is last-wins, per (a). If sqlite's `put-rows` recursion turns out to disagree with that,
   STOP and report; do not make mem match a sqlite behaviour you have not verified.
3. **If you find yourself editing `sqlite-store.wat`** — STOP. It is the reference.
4. **`row-in-delete-batch?` is O(rows × keys)** and a replace makes `put` pay the same cost.
   That is SCORE-stone-2 finding 3, already open, and it is **not yours to fix here** — but if
   the shape you write makes it worse than O(rows × batch), say so.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Read the Summary line. Floor at HEAD is 5096.
