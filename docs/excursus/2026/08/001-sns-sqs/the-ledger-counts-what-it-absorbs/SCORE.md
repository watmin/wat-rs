# SCORE — the ledger counts what it absorbs

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me on a quiet box.

```
Summary [ 365.629s] 5213 tests run: 5213 passed (4 slow), 15 skipped
FLOOR=0        .floor/2026-09-04T04-23-45Z/ (executor) · my own re-run green, 0 FAIL/TIMEOUT
```

## ★ THE NUMBER IS ZERO — and the tracker called it a week of stones ago

My five runs, rate 200 bp, seed 42, **not tuned**:

```
distinct=8000; dup=0; disrupts=24; seen-firsts=8000; seen-dups=0        ×5
```

**24 severs never interrupted a claim.** 3c exercised reconnection; the dedupe path never ran.
`dup=0` under chaos remains exactly as vacuous as it was — **and now we know that**, which we did
not before this stone.

★ **This was predicted, with its mechanism, before any of it was built.** The tracker's 3c entry:

> *"Client-side drops exercise reconnect; **they cannot produce a duplicate, because arms run to
> completion and an alarm fires between them.**"*

A `defservice` is a serializing actor: an arm runs to completion, so a disrupt alarm fires *between*
arms and can never land mid-claim. The measurement is that sentence, confirmed.

**That is the third prediction this campaign to hold, and all three named their mechanism up front** —
*durability makes the store hot* (1.29× → 1.68×), *the chain amortises by K but per-message CPU does
not* (3–5× predicted, 2.0×), and now this. Every prediction that stated only a number has died.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ the number that did not exist | ✅ **`seen-dups=0`**, five runs, untuned |
| 2 | ★★ the counter can fire | ✅ `redelivery_is_absorbed_by_the_consumer` PASS — deterministic redelivery gives `seen-dups=1` |
| 3 | ★ the two worlds print differently | ✅ `seen-dups=0` vs `seen-dups=1` — distinguishable lines |
| 4 | rate 0 unchanged | ✅ `total=8000; distinct=8000; dup=0; seen-firsts=8000; seen-dups=0; disrupts=0` |
| 5 | the worker untouched | ✅ `:326-330` unchanged |
| 6 | scope | ✅ `circuit.wat` + one test file |
| 7 | the floor | ✅ **5213/5213, my own re-run** |

## The disclosed delta was a STRENGTHENING, and it needed saying

The floor test exact-matched `"total=1;distinct=1;dup=0"` — a string that **cannot witness the
ledger**. It now reads:

```rust
assert_eq!(field(&stored, "total"), "1", …);
assert_eq!(field(&stored, "distinct"), "1", …);
assert_eq!(field(&stored, "dup"), "0", …);
assert!(seen_dups > 0,
    "the ledger must count the absorbed redelivery; a counter that never counts is a deleted counter");
```

All three original claims survive as field checks, `seen-dups > 0` is **added**, and the assertion
is now robust to the summary growing fields. My EXPECTATIONS forbade weakening an assertion; this is
the opposite, and it carries row 2's own sentence into the test where it will outlive this SCORE.

## ⛔ WHAT THIS DOES TO 3d — it is no longer a follow-up

Before this stone, 3d was *"produce a duplicate and show it absorbed"* — on the unmeasured premise
that duplicates were already occurring. **They are not.** So:

- **3d is now the FIRST fault that can make `seen-dups` move.** Not a confirmation of an existing
  path — the only thing that will exercise it.
- **Its acceptance criterion is exact**, and the tracker's own table already states it:

  | drop lands | work happened? | caller knows? | duplicate on retry? |
  |---|---|---|---|
  | before dispatch | no | no | no |
  | **after the arm, before the reply-send** | **yes** | **no** | **YES** |

  3c is row one of that table, measured. 3d is row two, and **`seen-dups > 0` is how we will know
  it landed** — a criterion that did not exist an hour ago.

★ **The negative result was worth more than the positive one would have been.** Had `seen-dups`
come back non-zero, we would have learned that chaos already exercised the dedupe and 3d was
confirmation. Zero tells us the fault domain has an unreached half, names which half, and hands 3d a
number to move. **That is why the instrument went first.**

## Still open

- **3d** — the reply-drop. `None` → `LOST` proven userland; `wat/service.wat` untouched. Now with an
  exact acceptance criterion.
- **S31** — `claimed` is `:ephemeral`; the dedupe ledger does not survive hibernation. Named, cut,
  untouched.
- **Stone D2** · **Stone C** · **S15**–**S31**.
