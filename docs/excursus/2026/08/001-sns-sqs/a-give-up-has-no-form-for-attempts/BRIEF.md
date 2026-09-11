# BRIEF — a give-up has no form for attempts

**Read `DESIGN.md` beside this first.** It contains the scope cut (three defects, not one class), one
thing you must REPORT rather than fix, and one thing deliberately out of scope.

## The work, in one paragraph

Give-up verdicts are `String`s and the consumer treats `""` as success, so a poller can silently pass
by returning nothing and can legally give up on an attempt count. Replace the `String` with a
`:fanout::Verdict` enum that has **no `Exhausted` variant and no empty-string success**, route
`require!` through it, and rebuild the two attempt-bounded pollers on one combinator whose signature
has no attempt parameter. Two illegal states stop having a form.

## The rooms

| where | why you are going there |
|---|---|
| `circuit.wat:1383` `require!` | the keystone. `[r <- String] -> nil`, `"" = ok`. Every verdict flows through it. |
| `circuit.wat:1750` `poll-until-drained*` | ⭑ THE PROVEN SHAPE — read before writing anything. Three outcomes, each naming its bound and numbers. Copy its semantics; replace only the String. |
| `circuit.wat:1634` `poll-until-filled*` | its twin, incl. the `>=`-not-`=` ruling at `:1520` — read that comment, it is the precedent for the sweep question. |
| `circuit.wat:2322` `join-publishers*` | 120 000 attempts; give-up is a bare `assertion-failed!` with **no numbers**. Worst of the four. |
| `circuit.wat:2348` `poll-until-visible-zero*` | 4000 attempts. Note `v == -1` is the **unreadable-tier sentinel** — it matters for the sweep question. |
| `circuit.wat:3649`, `:3737` | its two callers, **in the floor**. Your green proof. |
| `circuit.wat:1506` `sweep-drained?` | ⚠ REPORT ONLY — see STOP-2. |
| `circuit.wat:1711`, `:1580` | `drain-stale-polls` / `fill-stale-polls`, both 600, deliberately equal. Keep them equal. |

## Implementation sketch

```
(:wat::core::defenum :fanout::Verdict :wat::enum::Pure
  :Done    []
  :Stalled [no-progress-polls <- i64  elapsed-ms <- i64  snapshot <- String]
  :Ceiling [elapsed-ms <- i64  ceiling-ms <- i64  snapshot <- String])

(:wat::core::defn :fanout::require! [v <- :fanout::Verdict] -> :wat::core::nil
  (:wat::core::match v
    ((:fanout::Verdict::Done) nil)
    ((:fanout::Verdict::Stalled k ms snap) (:wat::kernel::assertion-failed! <formatted> …))
    ((:fanout::Verdict::Ceiling ms cap snap) (:wat::kernel::assertion-failed! <formatted> …))))
```

The message text `require!` raises should be **what the String verdicts say today** — the existing
wording is good and the floor's expectations may depend on it. You are changing the carrier, not the
report.

Then `join-publishers*` and `poll-until-visible-zero*` take `stall-k` + `ceiling-ms` instead of
`left`, and `poll-until-drained*` / `poll-until-filled*` return `Verdict` instead of `String`.

## Blast radius, measured

```
poll-until-drained 5 · poll-until-filled 5 · poll-until-visible-zero 6 · join-publishers 5
```
all in `wat-scripts/fanout/circuit.wat`. Confirm across `.wat`, `.rs` string literals and `.jsonl`
before claiming completeness — ⭑ count with `grep -o … | wc -l`, never `grep -c`, which counts LINES
and reported 1 where there were 11 earlier today.

## STOP triggers

1. **STOP-1 — if any site genuinely needs to report an attempt count**, STOP and say which. That is
   the contract decision of this gate (`Verdict` has no `Exhausted`, ever) and if it is wrong, the
   gate is wrong and I want to know before it ships, not after.
2. **STOP-2 — `sweep-drained?` is REPORT-ONLY.** Do NOT change `= 0` to `<= 0` without evidence. `-1`
   is the unreadable-tier sentinel (`poll-until-visible-zero*:2356`), so `<= 0` would read an
   unreadable tier as **drained** — strictly worse. Report which is correct and why; change it only
   if the evidence says so, and say so in the SCORE either way.
3. **STOP-3 — if `require!`'s new raise text changes what a floor test asserts**, STOP and report it.
   Do not patch a golden to fit (the previous stone reverted `src/freeze.rs` rather than patch one;
   copy that).
4. **STOP-4 — `owner-recv-loop` is OUT of scope.** If you find yourself editing `wat/service.wat`,
   stop: its bound is unreachable for a substrate reason (a bare `recv` never returns `TimedOut`) and
   it needs a deadline-bearing recv, which is a different stone.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the build does NOT compile
  tests, and that gap reddened the floor once already.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- Happy path: `./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000` →
  `distinct=8000;dup=0`.
- The chaos proof still green: `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- ⭑ **A negative control**: hand-construct a `Verdict::Stalled` and pass it to `require!`; it must
  raise and name its bound. A verdict type nobody can trip is theatre — this is the same check that
  caught `GaveUp` last stone.

## Shape to copy

`the-fill-poller-gets-what-the-drain-got/SCORE.md` — the last time these pollers were rebuilt, and the
source of the `>=`-not-`=` ruling. And `a-call-outcome-cannot-lie/` for replacing a lying return with
a parametric outcome enum.
