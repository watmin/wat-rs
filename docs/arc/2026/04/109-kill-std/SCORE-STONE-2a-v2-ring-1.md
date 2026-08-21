# SCORE — ②a v2, RING 1: 16 annotations, every one cited. The method holds.

Rider: 3.4 min, one flight, no STOP fired. Floor **4855/4855**, clippy **0**.
Blast radius exactly as scoped: `pass.wat` (14 rows) + `accum-pass.wat` (2 rows), nothing else.

| # | what | result |
|---|---|---|
| 1 | 16 annotations written, angle form | ✅ 16 insertions / 16 deletions, 1:1 |
| 2 | each cited to a caller's declared type | ✅ 16 rows, 16 citations |
| 3 | `network` / `facts` still bare | ✅ 12 / 6 |
| 4 | the bindings family still bare | ✅ `bindings` 6 · `ext` 2 · `km` 1 |
| 5 | only the two files touched | ✅ |
| 6 | build | ✅ 18.6s |
| 7 | floor | ✅ **4855/4855, 71.0s** |
| 8 | clippy | ✅ 0 |

## ★ The method that failed in v1 succeeded in v2, and the difference is checkability

v1 wrote types from **doc comments** — an authority that was stale and that the compiler could not
verify. It produced a nested `{join-bindings → …}` type that nothing constructs, a wrong `Value`, and
a 2620-failure floor.

v2 wrote types from **a caller's declared type** — an authority the compiler checks at every call
site. Same file, same roles, same rider tier: **green first try.**

> v1's method could not fail loudly. v2's cannot fail quietly.

## ★★ THE TRAP WAS REAL, AND THE RIDER WALKED UP TO IT AND STOPPED

The brief warned that `bm` is not evidence of beta-memory — pre-merge, two `bm` sites were local
*bindings* accumulators while 25 siblings were beta memory. The rider traced the body instead of
trusting the name, and I re-verified every step:

```
accum-pass.wat:49,59,68   (append-token bm node-id ntk)
pass.wat:83               append-token's first param is literally named `beta-mem`
```

So `bm` here **is** beta memory. And it found the actual bindings accumulator sitting fourteen lines
below in the same file:

```
accum-pass.wat:303        (fn [nb <- :wat::core::PersistentMap …])
```

`nb`, separately named, already on the brief's held-out list. **Both the trap and its decoy were in
one function, and the name-based reading would have swapped them.**

## ⚠ MY BRIEF NAMED THE FUNCTION WRONG — and the rider caught it

Row 15/16 said `accum-pass.wat:28` = `accumulate-pass`. Verified:

```
accum-pass.wat:28   (defn :wat::rete::accumulate-pass-for-token …)   ← the real fn at that line
accum-pass.wat:210  (defn :wat::rete::accumulate-pass …)             ← what I called it
```

**The line anchor was right; the name was wrong.** And `accumulate-pass` at :210 has no top-level `bm`
at all — its own param is `beta-mem`, still bare, and correctly outside these 16 rows.

★ The irony is the lesson: my brief's own contract says *"not from a name."* I then labelled a row
with a name I had not checked. The rider followed the contract rather than the label, which is the
only reason the label being wrong cost nothing.

## STOP-3 did not fire, and the rider proved it rather than assuming

It grepped every call site of all six functions across `wat/`. Five have **exactly one** caller each
(the four walkers). `accumulate-pass-for-token` has two, both in `accum-pass.wat` (`:267`, `:316`),
and the second is seeded from the first inside the same fold — same type, not a contradiction.

That is the check actually performed. Had it fired, it would have meant a second `walk-sorted-ids`:
a function polymorphic over the memory type, needing a split rather than an annotation.

## What ring 1 did NOT do

`network` (12 bare) and `facts` (6 bare) are untouched **because nothing upstream decides them** —
they are bare in the walkers too. They wait for a ring that reaches them; writing them now would have
been guessing wearing propagation's clothes.

The bindings family stays bare and unresolved. `Value` remains wrong (arc 278 R7), and whether the
merge's `$native`/`$oracle` split moved the consuming `<`/`>` sites is still **unmeasured**.

## Next

**Ring 2** — whatever the six now hand a typed memory to. `activate-fact` and `append-token` are the
known members; `append-token`'s `beta-mem` is bare at `pass.wat:83` and now receives a typed argument
from three separate ring-1 callers, so it is decided three ways over.
