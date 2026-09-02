# EXPECTATIONS — a pointer restored after the call is a pointer an unwind keeps

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,310 plus every arm you drive.**

## The scorecard, with pre-values measured at HEAD `35e0938cb`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ the strand | **(8,8) → (16,16) → (24,24)** across three panics (driven) | **(0, n) unchanged** across three |
| 2 | nested calls | outer `sp=4`, inner observed `sp=4`, arena stayed 4 — **the heap arm** (driven) | unchanged; the `Err` arm still taken |
| 3 | the false doc | `:96` *"Nested calls therefore stack rather than collide"* | gone; the ⚠ paragraph's true account stands alone |
| 4 | ⚠ `EXEC_SP`'s fate | inert — `start` always 0 | **decided**: deleted, or kept with the reason written |
| 5 | TLS teardown | — | `try_with` if needed, and **said** either way |
| 6 | hot path | — | a `Cell` write on drop; no allocation, no per-call branch added |
| 7 | radius | — | `expr_ir/eval.rs` + probe |
| 8 | lints | **196/196** (measured) | green |
| 9 | floor | **5310/5310** (measured) | ≥ 5,310, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **Remove the guard** → the three-panic probe REDs, showing the growth. *One* panic must not be
   enough to pass a partial fix — say what a single-panic probe would have done.
2. **Guard restores the wrong value** (`SpGuard(end)`) → REDs. Proves the probe reads the pointer,
   not merely "no panic escaped".
3. If `EXEC_SP` is kept: **make the `Ok` arm assert `start == 0`** and run the floor. If it holds,
   that is the evidence the mechanism is inert; if it fires, the drive was wrong and STOP-3 applies.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

40–55 minutes. The guard is six lines; the three-panic probe and the `EXEC_SP` decision are the work.

## What would make this strike a failure even if every test passes

**A single-panic probe.** A fix that resets the pointer once — or that happens to zero it on the next
entry — passes one panic and leaks on the second. Row 1 requires three, and mutation 1 must show the
growth.

The second: **keeping `EXEC_SP` unexamined.** The guard makes it provably dead. Leaving it as
"harmless bookkeeping" preserves a mechanism whose own doc is false, which is how this file got here.
