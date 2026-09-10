# DESIGN — one sample per boundary

## Why

Builder's objective: *"get it to the point where interpreted wat code and cold boot are the next real
targets."* `setup` is ruled out of scope as cold boot. That makes the in-scope budget, at
`2000 4 3 8192 true 1000`:

```
setup    12471 ms   ← COLD BOOT, out of scope        (55 % of the run)
collect   4583 ms   ← this stone
fill      2665 ms   ← next stone (p=1 has never been swept)
drain     2486 ms   ← drain-busy-ms=1465, so 1021 ms is WAITING
stop        298 ms
                    in-scope 10039 ms, of which only 1465 is measured WORK
```

★ **~8.6 s of a 22.5 s run is measurable waiting.** Remove it and the run is ~14 s, of which `setup`
is 89 % and the rest is interpreter work — exactly the state the objective names.

## The defect, counted

`Queue/stats` returns a **19-field `:queue::Stats`**. Five folds each make their own round-trip **per
queue** to read **one field**, and they are called repeatedly:

| fold | field it keeps | calls per run |
|---|---|---|
| `:fanout::sum-calls` | `receive-calls` | 1 |
| `:fanout::sum-ticks` | `ticks` | 1 |
| `:fanout::sum-store-calls` | `store-calls` | **4** |
| `:fanout::sum-store-ns` | `store-ns` | **3** |
| `:fanout::sum-handler-ns` | `handler-ns` | **2** |
| | | **11 round-trips per queue** |

At `m=4` that is **44 round-trips** to read five numbers. This is the third appearance of one defect
class today — after `:fanout::depth-of` discarding `acks` (fixed at `18a86fe27`) and `collect`'s folds
themselves. **One reply, many fields, N round-trips.**

## ⭑⭑ The instrument and the fix are the same change

`drain-busy-ms` is already `(hn-after − hn-before) / (1e6 × m)` — the queues' own `handler-ns` delta
across the phase. So **any phase gains a busy metric by sampling `handler-ns` at its boundaries**, and
that is the finish-line instrument the objective needs: *what fraction of this phase is work rather
than waiting.* Only `drain` has it today.

⛔ **But naively adding four more `sum-handler-ns` samples would add 4m round-trips and inflate the very
phases being measured** — the measurement containing the measurer. The only way to instrument every
phase *without* perturbing it is to make a boundary sample **cheap**, which is precisely the fix. Hence
one stone, not two.

## What it delivers

- `:fanout::Sample` — one record carrying `receive-calls`, `ticks`, `store-calls`, `store-ns`,
  `handler-ns`, summed across `qclients` from **one `Queue/stats` per queue**.
- Every one of the 11 call sites replaced by a sample taken at a **phase boundary**.
- **Per-phase busy-ms for `fill`, `drain`, `collect`, `stop`** from consecutive samples' `handler-ns`
  delta — the same arithmetic `drain-busy-ms` already uses.
- Round-trips per queue per run: **11 → the number of boundaries** (expected 3–5), at *lower* cost than
  today while measuring strictly more.

## The one contract decision

⛔ **The reconciliation identities must hold EXACTLY; the absolute values may shift by a sampling
instant.** Today's 11 round-trips read fields at 11 different moments, so the numbers reported are
already mutually inconsistent. One sample reads them at one instant, which is **more** correct — but a
value may differ from today's by a tick.

So the gate is on the identities, not the digits:
- `put + delete + count + scan` calls and ns **must** sum to `store-calls` / `store-ns`, remainder **0**
- `distinct = n×m`, `dup = 0`, inbox `accepted = n`
- and any absolute figure that moves must be **named in the SCORE with its direction and why**

## Out of scope = rejected

- **`fill`'s 2665 ms and the never-swept `p`.** The next stone; `run-p*` exists and is unused.
- **`drain`'s 1021 ms of waiting**, including the poller's own per-attempt round-trips — the observer
  effect cut from the stall stone. Still cut here; this stone touches boundaries, not the poll loop.
- **`stop`'s 298 ms** beyond gaining a busy metric.
- **`setup`.** Cold boot, ruled out by the builder.
- `sum-publisher-stats`, `sum-disrupts`, `seen-stats`, `collect-stop` — different peers, not `qclients`.
  Named so nobody assumes the sweep covered them.

## ⚠ What must not be claimed

⚠ This does not make the system faster **per message** — it removes harness round-trips. `drain` may
barely move; `collect` should move a lot. A SCORE claiming a throughput win is wrong.
⚠ And the new busy metrics are **server-side handler time**, not interpreter time. They bound how much
of a phase is *waiting*; they do not prove the remainder is interpretation. That distinction is the
whole point of the objective and must survive into the SCORE.
