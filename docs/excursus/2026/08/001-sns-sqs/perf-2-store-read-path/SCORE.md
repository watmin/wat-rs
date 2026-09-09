# SCORE — perf 2: the store's read path

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. One weigh.

```
Summary [ 342.589s] 5162 tests run: 5162 passed (3 slow), 15 skipped
FLOOR=0
```

| # | what | result |
|---|---|---|
| 1 | ★ cost flat in table size | ✅ **119 / 116 / 123 ms** (was 1691 / 3489 / 9204) |
| 2 | ★ the differentials hold | ✅ all pass, `git diff tests/` **empty** |
| 3 | the circuit faster and still correct | ◐ **PARTIAL — 257.7s vs 287.3s.** See below; the shortfall is my row's premise |
| 4 | durable Record unchanged | ✅ `:durable [rows <- (PersistentVector :- [StoredRow])]` byte-identical |
| 5 | hibernate/resume rebuilds | ✅ |
| 6 | `scan-index` got faster too | ✅ **119 / 105 / 108 ms**, flat |
| 7 | put-is-a-replace holds | ✅ (the differentials are its gate) |
| 8 | sqlite untouched | ✅ empty diff |
| 9 | header updated | ✅ |
| 10 | no runtime/surface change | ✅ |
| 11 | floor | ✅ 5162/5162, my own run |

Reads are now **O(result)**: ~75× at 1000 rows and, more importantly, **flat** — the table can grow
without the read cost following. The index lives in `:ephemeral`, rebuilt at `:init`, so the durable
soul and the hibernation format are untouched. The contract decision held exactly.

## ★ Row 3 is my error, not the strike's

My row said "materially under 287 s". It came back 257.7s — **10%**, which is not that.

But the shortfall is in the row's premise. I measured that **reads** were slow and then wrote a row
asserting that fixing them would materially speed the **circuit** — without ever measuring the
circuit's read/write split. Those are different claims, and only the first was measured.

The strike reported the gap plainly rather than hiding behind the 75× probe number, and named the
cause. I verified the mechanism rather than accept it: `mem.wat:516-531`, `put` is a **nested
foldl** — for each incoming row it folds the *entire table* to rebuild `kept`, then conj's. A
single-row put walks the whole table; `delete` (`:555`) the same. So writes are O(table) each and
O(n²) across a run, and with reads now flat **the circuit is write-bound.**

That is the honest reading of 257s: the stone did what it was scoped to do, completely, and the
bottleneck moved to the next thing.

★ **This is the same failure family as every earlier one in this campaign** — a row written from
inference rather than measurement. The three no-delta stones each avoided it by measuring first. Here
I measured one thing and asserted about another.

## Recorded, not chased

**sqlite runs the same circuit in 43s** — six times faster than mem, because it has a real database
underneath doing indexed writes. Recorded as context, explicitly **not** this stone's gate (STOP-4
and the DESIGN both put sqlite's performance out of scope), and not an argument for deleting
`mem-store`: mem is the zero-dependency backend and the differential oracle's other half.

## The next stone names itself

**perf-3: the store's write path.** `put`/`delete` are O(table) per call via the nested foldl, which
is now the circuit's dominant cost. The same `:ephemeral` index that made reads flat is most of what
a keyed write needs, and the same five differentials are the same oracle. It should be drawn with
the read/write split **measured first** — which is precisely what row 3 lacked.
