# HANDOFF — excursus 002 stone 1

You are striking one stone: **a `Peer` may not escape a scope that created its service's `Handle`.**

Start here, in this order:

1. `docs/excursus/2026/08/002-handle-lifetime-wall/DESIGN.md` — the rule, the four-row accept/reject
   table, the measured blast radius, and one collision that will otherwise surprise you mid-strike.
2. `BRIEF-stone-1-creation-scope-escape.md` beside it — the rooms as exact `file:line`, the sketch,
   the blast radius, and four STOP triggers.
3. `wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat` — green today. It holds both of
   your targets side by side: one function that must STOP compiling and one, nearly identical, that
   must KEEP compiling.

The single most important thing on this stone: **the discriminator is who CREATED the handle, not
whether one appears in the signature.** `(defn conn [h <- Handle] -> Peer …)` is safe — the caller
owns `h`. `(defn dial-and-drop [] -> Peer …)` is not — it starts the service itself and hands back
a channel to something that dies on return. A rule keyed on the parameter rejects every `conn`
helper in the corpus, including three in the stdlib.

Verify the census yourself rather than trusting the brief's numbers:

```
grep -rn --include=*.wat -E '\-> \(:wat::kernel::Peer' tests/ wat-scripts/ wat/ wat-tests/ examples/
```

Expected: 18 total, 2 that must be rejected, 16 that must keep compiling.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run it, name the exact arm, and surface it.

When you are done, write `SCORE-stone-1-creation-scope-escape.md` beside these: each EXPECTATIONS
row's real result, the honest deltas, the line counts. It will be graded by re-running, not by
reading.
