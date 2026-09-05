# EXPECTATIONS — STONE: the exploded form

## PART 1 — R11 always-breaks with a leading atom run

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ the one-line `do` EXPLODES | `claim-demo.wat`'s `(do (println "a") (println "b") n)` becomes one child per line |
| 3 | ★ **a leading ATOM rides** | fixture `(:wat::hashmap::assoc m (f b) (g b))` → `m` stays on the head line; both calls break |
| 4 | ★ **a leading COMPOUND means nothing rides** | fixture `(:wat::core::foldl (fn …) 0 xs)` → head line bare; `fn`, `0`, `xs` each own a line |
| 5 | idempotent | rows 2-4 all `IDEMPOTENT=true` |
| 6 | existing fixtures keep their ruled shape | `defn-multi`, `defn-empty`, `let-two`, `half-broken`, `all-four`, `unruled-*` — ruled + idempotent |
| 7 | the three walls still stand | a disagreeing-kind sabotage still raises; `ClaimedUnder` 0; `col` 0 in every rule file |

## PART 2 — `BlankBefore`

| # | what | expected |
|---|---|---|
| 8 | ★ a COMPLEX binder is followed by a blank line | `let` whose first binder's value is `(map (fn …) coll)` → blank line before the next binder |
| 9 | ★★ **simple binders get NO blank line** | `let-two.wat` (both values simple) → output UNCHANGED from today |
| 10 | ★★★ **blanks do not accumulate** | `fmt(fmt(x)) == fmt(x)` byte-identical on row 8's fixture |
| 11 | a blank never lands before the FIRST binder | the blank is *between* binders only |

## BOTH PARTS

| # | what | expected |
|---|---|---|
| 12 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 13 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 14 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 50-80 min. Part 1 is one `:when`; part 2 is a record, a map, emitter support,
a rule, and driver edits.

## Trap-doors named in advance

- **Row 3 is what makes row 2 non-trivial.** "Break every child" passes row 2 and FAILS row 3. A
  strike green on 2 and silent on 3 has not done part 1.
- **Row 9 is the failure that looks like success.** A blank after *every* binder passes row 8.
- **Row 10 is the one most likely to fail.** An emitted blank changes the next pass's input; the
  trigger must read STRUCTURE, never the previous output. STOP-3 forbids a collapse-consecutive-
  blanks hack as the fix.
- **Validate every probe fires before reading its silence.** The last stone cost three mis-aimed
  sabotages, and a file dropped in `rules/` is not loaded until a driver says so.
- **The vacuous green.** Row 12 prints the comment COUNT.
