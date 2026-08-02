# BRIEF — the `where`-expressivity axis: rows 2–5 against the proven row 1

Design: `DESIGN-expressivity-axis.md` (the three rulings). Seed fixture:
`wat-scripts/scratch-pad/probe-where-shape-spread.wat` (nine shapes; five could not compile before
`0d439a55` opened the purity fence).

**Row 1 is built, run and GREEN** — it is your exemplar and it already contains the extension
mechanism. Copy its shape; do not invent a new one.

```
#grid/Verdict {:axis "where-shapes" :size [200 1] :accuracy :match :runs 3 :ratio 9.9353 :winner :us}
```

## The question this axis answers

wat's `where` admits only **pure** functions; Clara's `:test` admits arbitrary Clojure. Does the
**same constraint, expressed in each, derive the same facts?** And the reason it matters: purity is
what makes every one of these predicates *compilable*, so this matrix defines the surface a
compiled-`where` executor (task #49a) has to cover. A shape we cannot express is a capability gap;
a shape we express but cannot compile is coverage data. Both are findings, neither is a failure.

## The work

Add four shapes to the existing axis. Each shape is **one grid cell** — the row index is part of the
size tuple (`size = [items row]`), so a `:MISMATCH` names the failing shape instead of hiding in a
unioned derived set.

| row | wat `where` | Clara `:test` |
|---|---|---|
| 2 | record accessor — `(:wat::core::i64::> (:wsh::Client/rep ?c) 0)` | `(> (:rep ?c) 0)` |
| 3 | String verb — `(:wat::core::String/starts-with? ?n "ad")` | `(clojure.string/starts-with? ?n "ad")` |
| 4 | collection verb — `(:wat::core::i64::> (:wat::core::PersistentVector/length ?t) 1)` | `(> (count ?t) 1)` |
| 5 | **a user-defined pure fn** — `(:wsh::big? ?k)` over a `defn` in the file | a `defn` in the generated ns |

Rows 3 and 4 are the fence, proven open. Row 5 is the one a compiled executor cannot model and must
hand back to the interpreter — it carries the whole #49a question, so do not skip or simplify it.

## Rooms, and why you are being sent there

| site | why |
|---|---|
| `wat-scripts/perf/grid/where-shapes.wat` — `:wsh::Req` | the shared fact stream; rows 2–4 need new fields |
| same — `rule-arith` | the row-1 exemplar: quasiquoted `conds` / `where-c` / `ins` folded into a `Rule`. Every new row is this fn with a different `where-c` |
| same — `build-rules` | the row dispatch; one `cond` arm per shape, `:else` already raises on an unknown row |
| same — `seed` | stages `Req(i)`; must stage the new fields |
| `wat-scripts/perf/grid/gen-where-shapes.sh` — the `case "$ROW"` block | the mirrored dispatch; one arm per shape, `*)` already exits non-zero |
| same — `defrecord Req` and the `seeds` binding | the Clara-side mirror of the stream |
| `wat-scripts/perf/grid/node-share.wat` | the axis exemplar row 1 itself was copied from, if you need a second reference |

## ★ Three rules that decide whether a row is worth anything

**1. THE SHARED CONDITION BINDS EVERY FIELD.** Change it once, to bind all of `?k ?c ?n ?t`, and
never again — every row then has every var available and adding a shape cannot perturb the token
stream. The axis's whole premise is that rows differ **only** at the trailing `where`.

**2. EVERY ROW MUST DISCRIMINATE — assert it, do not assume it.** A predicate that matches all facts
or no facts derives a set that is trivially equal on both sides, and its `:accuracy :match` proves
nothing (this is R59's vacuous-gate class, at the row level). For each row, check by hand from the
wat side alone that `0 < |derived| < items`, and put the expected count in the row's comment. If a
shape cannot be made to select a proper subset, that is STOP-2.

**3. SEED FROM A FORMULA OVER `k`, NEVER A DATA TABLE.** Both engines must compute the identical
stream independently; a literal table would have to be kept in sync by hand in two languages and
would rot. Derive each field from `k` — something in the spirit of `rep = k mod 5 - 2` (mixed sign),
`name = "ad"+k` when `k mod 3 == 0` else `"zz"+k`, `tags` of length `k mod 4`. Pick your own, but it
must be a formula, and the two sides must be visibly the same formula.

**4. MIRROR THE OPERATION, DO NOT IDIOMATISE IT.** Row 1 writes out `(- ?k (* (quot ?k 10) 10))` on
the Clara side rather than `(mod ?k 10)` deliberately, so the row measures the constraint and not a
translation choice. Where Clojure's semantics genuinely differ, write the note — `quot` rather than
`/` is already commented in the generator, because Clojure's `/` on two integers yields a **ratio**
and would silently change the meaning. If you hit another such case, comment it the same way.

## STOP triggers — rejection criteria. Ship nothing for that row; report the gap.

**STOP-1.** If a shape **cannot be expressed** in a wat `where` — it will not compile, or the purity
fence rejects it — STOP and report the exact form and the exact error. That is `:expressible false`,
the red row this axis exists to find, and it is a *result*. Do not route around it, do not weaken the
predicate until it compiles.

**STOP-2.** If a shape cannot be made to select a proper subset of the stream, STOP. A row that
matches everything or nothing is not a test of anything.

**STOP-3.** If making the two sides agree requires changing the *meaning* of the predicate on either
side, STOP and report both forms. A `:match` bought by bending the translation is worse than a
`:MISMATCH`, because it looks like evidence.

**STOP-4.** If adding fields to `Req` changes **row 1's derived set**, STOP. Row 1 is
`[3 13 23 … 193]` at `items=200` and must stay byte-identical — it is the regression check that the
seed change did not disturb the stream.

## Blast radius

`wat-scripts/perf/grid/where-shapes.wat` and `wat-scripts/perf/grid/gen-where-shapes.sh`. **Nothing
else.** No `src/`, no `wat/` stdlib, no new verb, no change to `run-axis.sh`.

## The gate — run these yourself, in the FOREGROUND, and report each verdict line

```
./wat-scripts/perf/grid/run-axis.sh where-shapes "200 1" "200 2" "200 3" "200 4" "200 5"
```

Five `#grid/Verdict` lines, every one `:accuracy :match`. Row 1 must still be `:match` (STOP-4).
Also paste, per row, the `:derived` from the wat side alone —
`echo '[200 <row>]' | ./target/release/wat ./wat-scripts/perf/grid/where-shapes.wat` — so the
non-vacuity check in rule 2 is visible rather than asserted.

This needs `target/release/wat` (already built) and clojure. It does not need cargo, so it will not
contend with anything the orchestrator is running.

## You are a rider, not the orchestrator

**Ending your turn ENDS you** — it does not suspend you, and nothing will wake you. There is no
notification coming. Run every command in the FOREGROUND and block on it; your turn ends when the
verdict lines are in your hands, not when a command is launched. Do not commit, do not push, do not
stash. Report what you changed, what you ran, what it said, and anything that surprised you.
