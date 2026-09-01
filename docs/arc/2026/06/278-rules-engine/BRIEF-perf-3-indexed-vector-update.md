# BRIEF — perf 3: the indexed vector update

Give `PersistentVector` an indexed `set` and a `drop-last`, then make the store's `put`/`delete` use
them instead of folding the whole table. **Measured**: 4.9 ms per put at 1000 rows and climbing
(3.33× → 3.67× per doubling), against 0.6 ms per scan, flat.

Read `DESIGN-STONE-the-indexed-vector-update.md` beside this first — the contract decision is that
**the defect is in core, not the store**, and two other routes were considered and rejected.

## Read in order, and why you are being sent there

1. **`wat-scripts/scratch-pad/probe-store-write-cost.wat`** — the measurement. **Re-run before and
   after**; its numbers are the stone's evidence and belong in the SCORE.
2. **`wat/query/mem.wat:516-531`** (`put`) and **`:555-563`** (`delete`) — the nested foldl. This is
   the consumer, and it is not the defect: it is the only shape a language without indexed update
   leaves available.
3. **`Cargo.toml:123`** — `rpds = "1"`. `PersistentVector` is a bit-partitioned trie; `set` is
   O(log n). You are **exposing** what is already there, not implementing a structure.
4. **`wat/query/mem.wat:499`** — the `:init` index rebuild, and the only other site touching
   `Record/rows`. Confirms the durable table's order is now irrelevant (no read path reads it).
5. **`src/collection/`** and the intrinsic registry — where a core collection primitive is registered
   and how its errors are located. Follow the house shape for bounds behaviour.

## The work

**1. Two core primitives** on `PersistentVector`: an indexed `set` and a `drop-last`. Out-of-range is
a **located error**, never a silent no-op — the house rule.

**2. The store's index carries positions.** `put`-replace becomes a `set` at the known index;
`put`-insert a `conj`; `delete` a **swap-remove** (move the last row into the hole, drop the last)
with the moved row's position fixed up in the index.

**3. Update `mem.wat`'s header** to describe the write path as it now is.

## Blast radius

The core primitive (Rust + its registration), and `wat/query/mem.wat`. **No `sqlite-store` change, no
durable Record shape change, no surface change, no `service.wat` change.**

## STOP triggers

**STOP-1 — the differentials are the gate.** Five mem-vs-sqlite tests. **Swap-remove is the change
most likely to trip them**: if any consumer depended on the durable table's order, one goes red.
That is the check working — STOP and report, never adjust the test.

**STOP-2 — order-independence is a claim, not an assumption.** The DESIGN says no read path touches
`Record/rows`. **Verify it yourself before relying on it.** If any site does, swap-remove is unsound
and the stone changes shape.

**STOP-3 — expose narrowly.** `set` and `drop-last`, because this consumer needs them. Not a general
`rpds` surface. A wide API with one consumer is an abstraction before its second user.

**STOP-4 — out-of-range must be loud.** A silent no-op on a bad index is exactly the swallow this
arc spent itself removing. Located error, with the index and the length.

## The gates to write

- **★ the cost:** re-run `probe-store-write-cost.wat`. Per-doubling must approach **2×**, not 3.7×.
  Baseline puts 400 / 1333 / 4887 ms, deletes 751 / 2801 ms.
- **★ the differentials:** all five pass, **unedited**.
- **the circuit:** re-run it. Same output string; report the wall time against 257 s. **Do not
  promise a number** — perf-2's row 3 asserted a circuit improvement from a read measurement and was
  wrong. Report what you measure.
- **the primitive's bounds:** an out-of-range `set` and `drop-last` on empty are located errors.
- **hibernate/resume** still rebuilds and answers identically.

## Prior comparable result

`SCORE-perf-2-store-read-path.md` — the stone before this, and its Row 3 section is why this brief
refuses to predict the circuit's number.
