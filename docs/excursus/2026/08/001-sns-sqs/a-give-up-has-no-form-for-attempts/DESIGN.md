# DESIGN — a give-up has no form for attempts

Builder: *"let's draw that gate"*, after *"making an illegal state not representable seems obvious"*.

**Drawn 2026-09-11. NOT STRUCK.** This is owed-ruling #1, and it is a **gate**, not four repairs.

## Why a gate and not repairs

Four sites were named as one class. Reading them, **they are three different defects**:

| | site | defect |
|---|---|---|
| **(a)** | `circuit.wat:2322` `join-publishers*` | attempt-bounded — `left <= 0` → `assertion-failed! "publishers never done"`, **120 000 attempts, and it prints NO numbers at all** |
| **(a)** | `circuit.wat:2348` `poll-until-visible-zero*` | attempt-bounded — 4000 attempts; its verdict does carry `attempts`+`elapsed`, but the BOUND is attempts. Two callers at `:3649`/`:3737`, **in the floor** |
| **(b)** | `circuit.wat:1506` `sweep-drained?` | a completion *predicate* with `= 0` on both terms — not a give-up at all |
| **(c)** | `service.wat:3815` `owner-recv-loop` | wall-clock bounded and names its bound, but its `TimedOut` arm **cannot fire** |

Only **(a)** is one class with one cure. (b) and (c) are handled explicitly below — **not** bundled on
resemblance, which is the mistake this campaign has made five times today.

## ⭑⭑ The gate, and it closes TWO holes because of what `require!` is

The two already-rebuilt pollers (`poll-until-drained*`, `poll-until-filled*`) are the proven shape —
three outcomes, each naming its bound and its numbers. Their one weakness is that the verdict is a
**`String`**, and the consumer is:

```
(:wat::core::defn :fanout::require! [r <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::= r "") nil (:wat::kernel::assertion-failed! r …)))
```

⛔ **`""` MEANS SUCCESS.** So a poller that forgets to return a verdict, or returns any empty-ish
string, **silently passes**. A missing case reads as OK — the same defect class as `Reply::Failed`
having no client arm, sitting in the consumer this time.

**Make the verdict a type:**

```
(:wat::core::defenum :fanout::Verdict :wat::enum::Pure
  :Done    []
  :Stalled [no-progress-polls <- i64  elapsed-ms <- i64  snapshot <- String]
  :Ceiling [elapsed-ms <- i64  ceiling-ms <- i64  snapshot <- String])
```

and `require!` takes a `Verdict`. That makes **two** states unrepresentable at once:

1. ⭑ **There is no `:Exhausted [attempts]` variant.** An attempt-bounded give-up has *nothing to
   return*. It cannot report "I ran out of tries", so it cannot be written that way — the same move as
   deleting `Failed` from the op reply enum, which the builder named as the obvious one.
2. ⭑ **`Done` is a variant, not `""`.** "Accidentally succeeded by returning nothing" has no form.

★ And *"must name which bound it hit"* stops being a convention: `Stalled` cannot be constructed
without its K and elapsed, `Ceiling` cannot be constructed without its ceiling. The type carries the
requirement the prose used to.

## The combinator

One helper the four pollers call, whose signature has **no attempt parameter**:

```
:fanout::poll-bounded :- [S]
  [probe <- (Fn [] -> S)          ;; one sample
   done? <- (Fn [S] -> bool)
   progressed? <- (Fn [S S] -> bool)
   stall-k <- i64                 ;; polls WITHOUT progress
   ceiling-ms <- i64
   snapshot <- (Fn [S] -> String)]
  -> (:wat::core::Tuple :- [:fanout::Verdict S])
```

`stall-k` counts *polls without progress* — a progress bound, not an attempt bound: a system that keeps
progressing never trips it, which is exactly the distinction `poll-until-drained*`'s own comment
records (*"an ATTEMPT budget expires on a busy box whether or not the system is healthy"*).

## Scope

**IN:** the `Verdict` type · `require!` migrated · `join-publishers*` and `poll-until-visible-zero*`
rebuilt on the combinator · `poll-until-drained*` and `poll-until-filled*` migrated to return
`Verdict` (they already have the shape; this removes the String).

**(b) `sweep-drained?` — REPORT, DO NOT BLIND-FIX.** It is listed as a "pinnable equality", and ⚠ **I
am not convinced it is wrong.** `= 0` on visible AND unacked, while `-1` is the *unreadable-tier
sentinel* used at `poll-until-visible-zero*:2356`. With `<= 0` an unreadable tier (`-1`) would read as
**drained** — strictly worse. The strike must report which is right **with evidence**, and change it
only if the evidence says so. A ruling, not a chore.

**(c) `owner-recv-loop` — OUT, with a named reason.** Its bound is real but unreachable: a bare
`:wat::kernel::recv` never returns `TimedOut` (never constructed as a value in Rust — verified in
`the-owner-faces-an-outcome/SCORE.md`). Fixing it needs a **deadline-bearing recv for owner handles**,
because `select` refuses to mix a `Thread`/`Process` handle with `after`'s `Peer`. That is a substrate
primitive and a separate stone. **This gate does not cover it, and must not pretend to.**

## The one contract decision

**`Verdict` has no `Exhausted` variant, ever.** If a future caller "needs" one, that is the defect
this gate exists to prevent, and the answer is a progress or wall-clock bound — not a new variant.

## Blast radius, measured

```
:fanout::poll-until-drained       5 occurrences
:fanout::poll-until-filled        5
:fanout::poll-until-visible-zero  6   (two callers in the floor: circuit.wat:3649, :3737)
:fanout::join-publishers          5
:fanout::require!                 the consumer; every verdict flows through it
```

All within `wat-scripts/fanout/circuit.wat`. ⚠ Confirm across **all three carriers** of wat source
(`.wat`, `.rs` string literals, `.jsonl`) before claiming completeness — that omission reddened the
floor twice in stone 1a.

## What this gate does NOT close

A new poller can still be hand-rolled with a decrementing counter and never call the combinator. The
type stops it from *reporting* an attempt bound, and the combinator makes the right path the easy one,
but nothing in the compiler forces a loop through it. **That is the highest rung this material
allows; saying so is part of the ruling.** A heuristic checker rule ("no self-recursive `defn`
terminating on a decrementing i64") would be fragile and is REJECTED.
