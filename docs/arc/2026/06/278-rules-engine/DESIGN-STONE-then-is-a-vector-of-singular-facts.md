# DESIGN-STONE — `:then` is a vector of singular facts

> **Status: DRAWN + RULED 2026-08-02.** Two stones under one ruling. Stone A (the surface
> migration) is executable now; Stone B (the empowerment) is drawn but not briefed — its shape
> depends on A landing.
>
> Sibling of `DESIGN-STONE-where-admits-only-rete-ops.md`, which does the same job for the LHS.
> That stone fences `:when`; this one fences `:then`.

## The ruling, in the builder's words

> *"static `:then [...]` … every member of that 'vec' must be a singular fact to insert… if you
> want many facts… then you write many fact creation statements."*

```clojure
(:wat::rete::defrule :ns::name
  :when [cond1 cond2 condN]
  :then [(f1 ?b ?n)
         (f2 ?m ?z)])
```

Both sides are vectors. Both sides are fenced. **Each `:then` member produces exactly ONE fact.**

## What is ruled OUT, affirmatively — not deferred

- **A dynamic fact count.** No splicing, no collection-valued member, no `(insert-all …)` in a
  `:then`. Ten facts means ten entries.
- **`do` as a grouping form.** `do` returns only its LAST value, so it would silently drop every
  fact but one — a silent wrong answer, the worst shape available under this arc's law.
- **Conditional insertion inside an entry.** If a fact should sometimes not be deduced, that is a
  second rule with a different `:when`.

### ★ Why the static count is a WEAPON, not a limit

Each firing's output becomes **statically known** — finite, countable, and inspectable before the
engine runs. That is precisely what `#49a`'s compiled `where` wants from the RHS, and it is the same
shape as the two scope reductions that already bought this engine its edge:

| reduction | what it buys |
|---|---|
| inserts-only (the mutating bangs cut) | truth maintenance falls out of replay — R2 |
| pure RHS | the snapshot is a THUNK; store `{facts, rules}`, re-derive on demand — R5 |
| **static fact count per firing** | **the RHS output is finite and knowable at compile time** |

Clara can `insert!` in a loop, so its RHS output is unbounded and statically unknowable. Choosing
not to is the same trade, made a third time, deliberately.

## Stone A — the surface migration (executable now)

`:then <i1> <i2> …` → `:then [f1 f2 …]`, **and drop the `(:wat::rete::insert …)` wrapper.**

**Dropping the wrapper is the second win and it is not cosmetic.** The engine is inserts-only by
doctrine, so *every* `:then` entry is an insert — naming it per entry says nothing. And it retires a
name doing two jobs: `:wat::rete::insert` currently means `(insert session fact)` at the session
level (a `defclause`, `rete.wat:1004`) **and** `(insert fact)` as an RHS marker with no session
(`matcher.rs`'s `build_insert_fact`, which validates that exact head). Drop the RHS meaning and one
name means one thing.

**The macro gets SMALLER.** Today it splices varargs (`then-forms = (rest (rest (rest rest)))`,
carrying a long comment about `drop` having gone lazy and the F5 pure-total allow-list excluding
`to-vec`/`into`). The new form is `then-vec = (get rest 3)`, quoted as-is — exactly symmetric with
`when-vec = (get rest 1)`. The gymnastics and their comment delete.

**Scope, measured 2026-08-02:** 197 `defrule` sites across 54 files — `tests/rete` 19,
`wat-scripts/scratch-pad` 11, `wat-scripts/perf/grid` 11, `wat-scripts/fixes` 9, `wat/` 2,
`tests/services` 2.

**This is a `wat-fix` codemod** (CLAUDE.md item 1), not hand-edits and not python.

**The bootstrap order matters — 2 of the 54 files are STDLIB.** Changing the macro breaks its
callers, and two callers live in `wat/`, so the stdlib would fail to load and the runtime could not
run the codemod. Therefore: change the macro **and hand-migrate those 2 stdlib files in the same
step** (that is the bootstrap, exactly what `wat/fix.wat`'s STASH-DANCE header covers), get a
loadable runtime, then codemod the other 52. Mid-strike red is expected and fine; the commit is
atomic.

*Fallback if that does not yield a loadable runtime:* widen the macro to accept both forms, codemod,
then narrow. Costs a temporary second spelling and needs an AST-kind test the F5 allow-list may not
permit — which is why it is the fallback, not the plan.

**Stone A changes NO semantics.** Same facts, same order, same everything — only the surface. That
separation is load-bearing: if A and B land together, a failure cannot be attributed.

## Stone B — the empowerment (drawn, not briefed)

Each `:then` member may be a **fenced expression that produces exactly one fact**, instead of only a
literal record constructor.

```clojure
:then [(:usr::make-rate ?count ?window)]
```

- **Fenced by pure ∧ deterministic ∧ total** — the same axes as `:when`. Confirmed available: the
  fence already operates on **quoted AST** (`pure?` takes a quoted expr, and a rule body is quoted
  by `make-rule`), so this is `classify_expr`/`head_ok` pointed at a second site — exactly what the
  accumulator branch already does at `rete.wat:791`.
- **Under the ratified DSL-closure ruling**, heads must be `:wat::rete::`-namespaced, with the
  accessor/constructor and `sym.functions` composition doors intact.
- **The composition door is what makes it work.** `(:usr::make-rate ?count ?window)` is admissible
  because `classify_fn` walks the user fn's body transitively and it bottoms out in rete ops. That
  is the "basis, not ceiling" property finally used on the RHS.
- **Mechanism:** `build_insert_fact`'s operand resolution widens from `resolve_operand`
  (`matcher.rs:516` — `?var` / `:field` / literal only, nested lists explicitly refused as
  *"where-territory"*) to a fenced evaluation.

**It also closes compute-and-bind.** A `where` mints a TestNode, which FILTERS tokens — it does not
bind, so `?rate` could never be computed in the LHS and reused. With `:then` calling user fns, the
arithmetic lives inside the fn. No binder form is needed.

### ⛔ B DOES NOT SHIP WITHOUT THE FENCE

R5's *"the snapshot is a thunk"* advantage — the one Clara structurally cannot have, because its
impure RHS forces it to STORE derived state — rests entirely on the RHS being pure. Widening `:then`
to arbitrary expressions **without** the fence would trade this engine's central architectural edge
for convenience. The fence is what makes the widening safe; they are one stone, not two.

## Open, to ground before B is briefed

1. Is `resolve_operand`'s no-eval contract also a **performance** contract? It runs per derived
   fact, and `#43` already fought that path. **Measure before widening a hot loop.**
2. Interaction with `#49a`'s compiled RHS.

## Order

**A → B.** A is mechanical and independent; B rests on A's surface. Neither is blocked by `#57`, but
B should follow it, since arming the `where` fence settles the vocabulary B reuses.
