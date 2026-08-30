# ⛔ NOTE (arc 301) — `mem-store`'s `put` APPENDS where `sqlite-store`'s REPLACES; the ORACLE admits a state the real backend cannot represent

**Found 2026-08-30 by probe, while drawing stone 3.** Pre-existing — **not** introduced by
stone 2 or 2b. Verified: `mem.wat`'s `put` was already a bare `conj` at `fe1e923d5^`.

## The measurement

Put `(pk "q#1", sk "a")` projecting `by-v → "v1"`. Then put **the same `(pk, sk)`** projecting
`by-v → "v9"`. Scan the base table and the GSI over a range covering both:

```
MEM[base=2:a,a;gsi=2:v1,v9]     SQLITE[base=1:a;gsi=1:v9]
```

- **sqlite** — one row, projection MOVED. `put-one-row` is
  `DELETE main → clear-index-projections → INSERT main → insert-index-projections`, so a
  re-put of an existing key is an idempotent **replace** that carries its index with it.
- **mem** — **two rows sharing one `(pk, sk)`**, and BOTH projections alive. `put` is
  `foldl … (:wat::vector::conj acc r)`. It appends. There is no key.

## Why this is worse than "two backends differ"

`sqlite-store`'s own DDL:

```sql
CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL,
                                 PRIMARY KEY(pk,sk))
```

**`(pk, sk)` is a PRIMARY KEY.** Duplicate keys are not merely undesirable in the real backend
— they are *unrepresentable*. And `mem-store` is the **oracle**: the thing other tests compare
sqlite against.

> **An oracle that admits states the subject cannot represent is not an oracle.** It will
> certify behaviour that cannot occur, and — as here — stay silent on behaviour that differs.

`wat/query.wat`'s Store surface never says which one is right. `put`'s doc-comment is *"write a
batch ATOMICALLY (one transaction)"* — it says nothing about key semantics, so both
implementations are defensible readings of the contract. **The contract is the defect.**

## Why stone 2b did not catch it

2b's differential exercised `ensure-schema → put(3 distinct keys) → delete → scan → scan-index
→ delete-again → scan → scan-index`. **It never re-put an existing key.** Delete agreed
perfectly, which is what 2b was drawn to measure and what it correctly reported.

The lesson is not that 2b was weak — it is that **a differential proves agreement only over the
operation sequences it actually drives.** A green differential is a statement about a path, not
about a pair of implementations.

## Why it surfaced now

Stone 3's queue design turns on exactly this operation. The best available design makes a
message invisible by **re-putting its row with a later `visible-at` index-key**:

- `pk` = queue, `sk` = a STABLE message id
- GSI `by-visible-at`: `ipk` = queue, `isk` = when it becomes visible
- `receive` → `scan-index` where `isk <= now`, then re-`put` the same row with
  `isk = now + timeout`

That is strictly better than the arc DESIGN.md sketch (`sk` = visible-at), because a stable
`sk` means **`ack` names the same key forever** — no receipt-handle drift — and making a
message invisible is ONE atomic `put` rather than put-at-new-key + delete-at-old-key, which has
a crash window that would **duplicate the message**.

**On sqlite that design works. On mem it silently duplicates every in-flight message.**

## What is owed — stone 2c, before stone 3

1. **Rule the contract.** `put` is a replace-by-`(pk, sk)`, or it is an append and the surface
   says so. The DDL has already voted; the surface should say it out loud.
2. **Make `mem-store` match.** Its `put` folds the incoming batch onto `Record/rows`; a
   replace is that fold with the same `key-hits-row?` predicate stone 2 already wrote for
   `delete` — drop any existing row whose `(pk, sk)` a new row names, then `conj`.
3. **Extend the differential to cover re-put**, so this class cannot return. The probe above
   is the fixture, minus the standalone `:user::main`.

## Kin

- `SCORE-stone-2b-delete-differential.md` — the differential this escaped, and why.
- `wat/query/sqlite-store.wat:~200` `put-one-row` — the clear-then-insert that makes sqlite a
  replace.
- `wat/query/mem.wat` `put` — the `conj` that makes mem an append.
