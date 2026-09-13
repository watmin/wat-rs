# EXPECTATIONS — accepted is a count

**Written BEFORE the strike**, from `bb993ffd5`.

## What this stone is graded on

That `Accepted n` is a **count**, verified against the store's own contents while the reply was destroyed.
The floor cannot give this — nothing in the corpus loses a store reply — so the proxy-driven probe is the
proof, the fourth stone running where the default gate measures something else.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ `n` **equals** the store's truth | proxy with `drop-reply-bp` live | **`n == real-rows`**, an equality — not `n > 0`, not `0` |
| 2 | the write landed while the reply died | same probe | `real-rows > 0` |
| 3 | the count is an INTERSECTION | read the diff | `scanned ∩ rows.sk`, **never** `length scanned` |
| 4 | paging handled or proven unnecessary | read the diff | cursor followed, **or** a stated proof that one page covers `take` |
| 5 | all three arms | read the diff | `Lost`, `Closed`, `TimedOut` — not one of three |
| 6 | probe-also-fails ⇒ raise | read the diff | a dead-store raise, **no** new variant, **no** `Accepted 0` fallback |
| 7 | ⛔ no happy-path cost | `store-calls` on `2000 4 3 8192 true 1000` | **unchanged** |
| 8 | ⛔ ack side untouched | `git diff` | `:1215`/`:1248`/`:1279` unchanged |
| 9 | ⛔ neither caller touched | `git diff -- wat-scripts/topic/sns-fanout.wat` | **EMPTY** |
| 10 | ⛔ no new variant | `git diff -- wat/` | **EMPTY** — `SendResponse` unchanged |
| 11 | D2's gate does not regress | `probe_chaos_gate_has_teeth` | passes. ⚠ It covers the **disrupt** injector, **NOT** this path — no regression signal here; row 1 is the only evidence |
| 12 | floor | `./scripts/floor.sh` → **Summary line** | green at **5239** (+ any new test), 0 FAIL, no `ARM.txt` |
| 13 | clippy | `-D warnings` | exit 0 |
| 14 | happy / chaos | the two commands | `distinct=8000;dup=0` · exit 0 |
| 15 | ⚠ `rt-store` does NOT improve | happy `rt-store` vs ~6450–6630 | **same band** — the −17.1 % is unblocked, not banked |
| 16 | ⚠ any new test sized against the WALL | its runtime vs ~40 s under contention | stated as headroom, not just seconds |

## ⭑ Row 1 is an equality on purpose

`n > 0` would pass while `n` was wrong, and D2 just proved a threshold's flake rate depends on parameters
another stone may move (`0.6 %` → `~85 %` when n dropped). **`n == real-rows` cannot be satisfied by a lucky
number.**

## ⚠ Two nulls I commit to in advance

- **Row 7** — `store-calls` on the unperturbed run must be **unchanged**. The scan is on the failure path; if
  the happy path pays for it, it leaked.
- **Row 15** — `rt-store` must **not** move. If it does, either the scan leaked (row 7) or the scope was
  misread.

## ⛔ What I will reject

- **`length scanned` as the count** (row 3) — plausible, larger, wrong. The single most likely silent defect.
- **A truncated first page** (row 4) — plausible, smaller, wrong.
- **`Accepted 0` kept as a fallback** when the probe fails (row 6). That reinstates the overloaded field in a
  smaller window and leaves the defect representable.
- **A new `SendResponse` variant** (row 10 / STOP-5) — D1-c was ruled over D1-b.
- **Either caller edited** (row 9). They must become correct *without* being touched; that is the evidence.
- **`5237` quoted as the floor count** (row 12). D2 moved it to 5239.

## Runtime prediction

**45–70 minutes.** Three arms sharing one helper is the bulk; the proxy rig already exists. Floor ~500–560 s.

## Trap-doors, ranked

1. ⛔ **`length scanned` instead of the intersection** (row 3).
2. ⛔ **Cursor ignored** (row 4).
3. **Scan leaking onto the happy path** (rows 7, 15).
4. **Only one or two of the three arms** (row 5) — the sibling failure, five times in this campaign.
5. **A new test that times out under floor contention** (row 16) — cost this campaign a red floor one stone ago.
