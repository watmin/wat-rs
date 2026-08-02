# BRIEF — grow the `where`-expressivity corpus, one family per rider

## Why this exists, and it is not a benchmark

The builder's intent (2026-08-01): **wat programs will build primarily in rete — complex logic gets
offloaded to rules.** If that holds, a `where` clause is not a filter you occasionally write; it is
where user logic *lives*. And because a `where` admits only PURE functions, every one of those
predicates is COMPILABLE — that is the whole premise of task #49a, the compiled-`where` executor.

So this corpus has two jobs, and the second one is the reason to go wide:

1. **Hard references for the compiler.** Every shape that lands here is a form #49a must model.
   A corpus of six shapes yields a compiler that handles six shapes.
2. **Find where rete is wrong or broken.** Every row is a live differential against Clara. A
   `:MISMATCH` is a real defect in one of the two engines. A shape that will not compile is a real
   boundary. Both are findings; neither is a failure of your strike.

## Your exemplar — copy it, do not invent a shape

`wat-scripts/perf/grid/where-shapes.wat` + `where-shapes.clj` (six rows, green, byte-identical).
Read BOTH before you write anything. Your pair is that pair with a different family of predicates.

## The work

Create **`wat-scripts/perf/grid/where-<FAMILY>.wat`** and **`where-<FAMILY>.clj`** — a self-contained
pair holding **8–15 rows** of your assigned family. Own records, own seed, own rules; you share
nothing with any other pair, which is exactly why you cannot collide with another rider.

**GO COMPLEX.** The corpus already covers the easy shapes. What is missing — and what the compiler
needs — is depth: predicates that nest, compose, chain, and combine. A row that is one operator
applied to one binding teaches the compiler nothing it does not already know from row 1. Reach for
the forms a user writing real program logic would actually write, and then reach past them.

## The four rules that decide whether a row is worth anything

1. **THE SHARED CONDITION BINDS EVERY FIELD.** One leading `[Req …]` pattern binding all your fields,
   identical in every rule; only the trailing `where` / `:test` differs. Set it once. A row must only
   ever perturb its own predicate, never the token stream the family shares.
2. **EVERY ROW MUST DISCRIMINATE A PROPER SUBSET** — `0 < |derived| < items`. A predicate matching
   all facts or none derives a set that is trivially equal on both sides and proves nothing (R59's
   vacuous-gate class at the row level). Put the expected count in the row's comment, derived from
   the seed formula, and **check it against what the row actually emits.** If they disagree, one of
   the two is wrong and you must find out which before shipping the row.
3. **SEED FROM A FORMULA OVER `i`, NEVER A DATA TABLE.** Both engines compute the identical stream
   independently. A literal table would need hand-syncing in two languages and would rot.
4. **MIRROR THE OPERATION, DO NOT IDIOMATISE IT.** Write the same arithmetic on both sides rather
   than swapping in a Clojure idiom, so the row measures the constraint and not a translation choice.
   Where the languages genuinely differ, put the note in the comment — e.g. Clojure's `/` on two ints
   yields a RATIO, so the Clara side uses `quot` to mirror wat's truncating `i64::/`. That is
   faithfulness, not a fudge.

## STOP triggers — these are REJECTION criteria and also the most valuable output you can produce

**STOP-1 — A SHAPE THAT WILL NOT COMPILE IS THE POINT, NOT A PROBLEM.** If a form is rejected by the
checker or the purity fence: **STOP on that row, record the EXACT form and the EXACT error verbatim,
and move to the next row.** Do NOT weaken the predicate until it compiles, do NOT route around it,
do NOT quietly drop it. A rejected shape is either a genuine capability boundary the compiler need
not handle, or a fence bug — and we cannot tell which if you smooth it away. **Collect every one of
these; they are the headline of your report.**

**STOP-2.** If a shape cannot be made to select a proper subset, drop the row and say why. A row that
matches everything or nothing tests nothing.

**STOP-3.** If making the two sides agree requires changing the MEANING of the predicate on either
side, STOP and report both forms. A green bought by bending the translation is worse than a red,
because it looks like evidence.

**STOP-4.** If a `:MISMATCH` survives — the two engines genuinely disagree on the same constraint —
**that is a defect and it is the best thing you can find.** Do not tune the row until it matches.
Report the row, both derived sets, and your reading of which side looks wrong.

## Your gate — run it yourself, in the FOREGROUND

```
./wat-scripts/perf/grid/check-where-shapes.sh where-<FAMILY>
```

Green means your pair's rows are byte-identical across both engines. While iterating, the fast
per-file arbiter is `./target/release/wat --check wat-scripts/perf/grid/where-<FAMILY>.wat` (~0.2s) —
it names the exact wrong form and the exact line, and one located error at a time is the fastest way
to learn the language's real shape.

**No cargo. Nothing you touch requires a build** — the `wat` binary and clojure are already there,
so you will never contend with another rider.

## Blast radius

**Your two new files only.** Do not touch `where-shapes.*`, any other family's pair,
`check-where-shapes.sh`, anything under `src/`, `wat/`, or any other axis. Do not commit, do not
push, do not stash, do not revert anyone else's work.

## Report

- the rows you landed, with each one's count
- **every STOP-1 verbatim** — the form and the error (the headline)
- any `:MISMATCH` you could not resolve, with both derived sets
- anything that surprised you about what the `where` position does or does not admit

## You are a rider, not the orchestrator

**Ending your turn ENDS you** — it does not suspend you, and nothing will wake you. There is no
notification coming. Run every command in the FOREGROUND and block on it; your turn ends when the
gate's output is in your hands, not when a command is launched.
