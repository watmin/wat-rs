# EXPECTATIONS — a read declares its page

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ **No timing is gated, and none is even expected to move.** This stone bounds responses and adds
a tool; the measured 2.4× is a *different* item that this stone only makes safe to address. If
`publish` moves at all, that is a finding to explain, not a win to claim.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the bound truncates** | probe: 250 rows stored, `scan-index :limit 1000` | **≤ 64 rows** returned and a `Some` cursor. `limit` is a request, not a promise |
| 2 | ★★ **`-all` yields everything, in order** | `scan-index-all` over the same 250 | all **250**, in index order, across 4 pages, ending on a `None` cursor |
| 3 | ★★ **it is lazy** | read the impl; probe pulls only the first page | the second page is not fetched until the stream is consumed past 64. An eager vector re-creates the unbounded response this stone exists to prevent |
| 4 | ⛔ **`receive` is bounded and gets NO tool** | `Queue::receive :limit 1000`; `grep receive-all` | ≤ 64 envelopes; **no `receive-all` emitted anywhere** |
| 5 | ⛔ **emission condition holds both ways** | `grep` the expansion | `scan-index-all` and `scan-all` exist; **no `receive-all`, no `stats-all`, no `send-all` on a read** |
| 6 | ⛔ **defs are emitted and readable** | evaluate `:queue::Queue::RECEIVE-MAX-PAGE` | `64`, as `-MAX-ENTRIES` already does |
| 7 | ⛔ **no caller's `:limit` changed** | `git diff` on `circuit.wat`, `sns-fanout.wat`, `sqs.wat` call sites | untouched. **Especially `sqs.wat:170`** |
| 8 | ⛔ **delivery still exact** | `circuit.wat` ×3 at m=4 | `total=8000; distinct=8000; dup=0` |
| 9 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 10 | ⛔ **blast radius** | `git status --porcelain` | the six briefed files + probes + SCORE. **No `circuit.wat`, no `sns-fanout.wat`** |

## REPORTS

| ▪ | what |
|---|---|
| a | `publish` median ×3 at m=4 — **expected unchanged at ~18400**. Any movement is a finding |
| b | pages walked by `scan-index-all` for 250 rows at bound 64 |
| c | whether truncation had to live in each impl or could be applied at the boundary (STOP-3's answer, even if it did not fire) |

## RUNTIME

60–90 min. Two declarations' worth of parse, a def pair, an emission condition, and a lazy pull
loop. The `Stream` is the part with the least prior art in this tree.

## TRAP DOORS

- ⚠⚠ **Truncate, do not reject.** `:max-entries` rejects an over-limit request; `:max-page`
  **serves short**. A reader will expect symmetry and be wrong. Rejecting a `:limit 1000` would
  break every existing caller, which is the opposite of a bound that makes big reads safe.
- ⚠⚠ **Row 3 is the one that matters most.** A `-all` that eagerly materialises every page has
  *re-created* the unbounded response with extra steps — it would pass rows 1 and 2 and defeat the
  stone entirely.
- ⚠ **`receive` must not get a tool.** Its reads are leased: an `-all` drains rather than pages.
  Rows 4 and 5 gate this from both directions.
- ⚠ **`sqs.wat:170` stays `:cursor None`.** Making `take` follow the cursor is a behaviour change
  to the queue and is not this stone; row 7 gates it.
- ⚠ **Do not claim the 2.4×.** This stone makes raising a `:limit` *safe*; it does not raise one.
