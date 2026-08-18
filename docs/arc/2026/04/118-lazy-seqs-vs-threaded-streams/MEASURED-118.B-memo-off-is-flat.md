# MEASURED — memo-off is **FLAT**. The shared premise holds, and the memo costs TIME too.

**2026-08-17, against `5069fd05`.** Decision 4's probe, run before any of stone B is built.
Throwaway build with both memos bypassed in `realize`; substrate restored and rebuilt afterwards
(`git diff src/stream/mod.rs` → empty).

## The intervention control — run FIRST, because a no-op edit looks exactly like success

`wat-scripts/scratch-pad/probe-118B-memo-state-detector.wat` — a printing `f`, five elements,
drained by `into` (which walks three-call via `stream->pvec`):

```
memo ON  (HEAD binary)      "FORCED" × 5      ← one force per cell
memo OFF (throwaway build)  "FORCED" × 15     ← three forces per cell
```

**15 for 5 is the intervention proving itself** — and it is the three-call defect made visible.
Without this control, a build where the edit silently did nothing would have produced memory numbers
indistinguishable from a successful bypass. `[[feedback_a_green_test_can_prove_nothing]]`

## ★ POPULATION C — the wat-closure generator (the builder's Ruby-Enumerator idiom)

`probe-118B-dorun-retention-slope.wat`: unbounded generator → `take n` → `reduce +` into one `i64`.
Retains nothing by construction. Non-vacuity is the exact sum `n(n-1)/2`, **verified OK at every
point in both columns** — identical work, identical results.

```
n            memo ON        memo OFF
100,000      356,380 KB      44,616 KB
200,000      675,096 KB      44,228 KB
400,000    1,312,572 KB      44,400 KB
800,000    2,587,964 KB      44,512 KB
────────────────────────────────────────
slope      3,188 B/element   ~0 — FLAT (−0.15 B/elem across an 8× range; noise is ±400 KB / 0.9%)
at 800k                      58× smaller
```

**O(n) → O(1).** The premise every route rests on is **CONFIRMED for the population that had never
been tested** — and this is the one that matters, because it is what a user writing a producer in
wat actually gets.

## POPULATION B — the native `map` chain, n = 400,000

```
                        memo ON              memo OFF
A  floor (no stream)     63,120 KB  0.69s    62,668 KB  0.71s   ← control: did NOT move
B  native map chain     200,284 KB  3.99s    87,816 KB  1.70s
────────────────────────────────────────────────────────────
B − A                   137,164 KB           25,148 KB
per element                 343 B                63 B
```

The residual 63 B/element is **the materialized `range` source**, which is legitimately retained —
population B's source is a real container. Population C has no materialized source, which is why C
goes fully flat and B does not. Consistent, and the A control not moving (0.69 → 0.71s) rules out
machine state.

## ★★ THE UNEXPECTED RESULT — the memo costs WALL CLOCK, not just memory

```
population B, n=400,000     3.99s  →  1.70s     memo-off is 2.35× FASTER
population C, n=800,000     7.06s  →  7.59s     memo-off is 1.08× slower
```

**The memo is not a performance optimization. On the native path it is a 2.35× *slowdown*** — its
per-cell `OnceLock` + `Arc` allocation and the allocator pressure of holding 2.5 GB live cost more
than simply re-running a cheap native closure three times.

The divergence between B and C is explicable and worth keeping: **re-forcing a WAT closure is
expensive** (three interpreted `apply_function` calls per cell — population C), **re-forcing a
NATIVE closure is cheap** (population B). So the memo's time value depends entirely on the thunk
kind, and it is negative on the native path.

**Under `next` — one force per cell — neither population pays either cost.**

## What this settles

- **The premise holds.** Deleting the memos takes retention to O(1) for the generator idiom.
- **The memo's ONLY function is masking the three-call defect.** It buys nothing else: it costs
  memory always, and time on the native path. It is a patch, and it is a patch with a price on
  both axes.
- **The order remains load-bearing, and now for a measured reason.** The memo cannot die while any
  three-call walker survives — the control above shows exactly what happens: **user code runs three
  times.** Migrate first, delete second.

## What it does NOT settle

- **These numbers are memo-off with the three-call walk still in place**, i.e. `f` running 3×. The
  post-stone-B state (one force AND no memo) is not directly measured here — but it is bounded on
  both sides: memory by the flat column above, force-count by `next`'s own proven one-force-per-call
  (118.11a row 3).
- `/usr/bin/time` peak RSS, one run per point, one machine. **The slope is the claim**; the absolute
  footprint is not portable. `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
- Wall-clock here is a **single run per cell**, not a distribution. The 2.35× is large enough to
  survive that, the 1.08× is not — treat C's wall as "no meaningful change", not as a regression.
