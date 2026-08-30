# ⛔ NOTE — `foldl` should have been `reduce` the whole time

> **Builder, 2026-08-30:** *"can you add an arc 109 NOTE to kill foldl and just name it reduce? we
> dropped foldr not long ago… i think we should have just called it reduce the whole time."*
>
> Filed for arc 109 (the clojure-ification). **Not scoped, not drawn.** Measured this session so a
> later stone starts from evidence.

## What is actually on disk — measured, not remembered

```
:wat::core::foldl    1 dispatch arm (native Rust) · 2 wat defns (the ORACLE pair) · 572 corpus calls
:wat::core::reduce   1 wat DEFCLAUSE (wat/seq.wat:317)                             ·  38 corpus calls
:wat::core::foldr    GONE — no dispatch, no defn, no defclause
```

## ★ `reduce` IS ALREADY THE CLOJURE SURFACE OVER `foldl` — and it says so

`wat/seq.wat:317`:

```clojure
(:wat::core::defclause :wat::core::reduce
  ;; 3-arity: explicit init. Straight to the native `foldl` — no normalisation, no walker.
  ([f <- [U T :-> U] init <- :U coll <- (:wat::core::Seqable :- [T])] -> :U
    (:wat::core::foldl f init coll))
  ;; 2-arity: no init — the first element seeds the fold. Empty raises, by name.
  …(:wat::core::foldl f value rest)…)
```

**Both clauses bottom out in `foldl`.** `reduce` adds exactly one thing the native lacks: the
2-arity form that seeds from the first element — which is Clojure's own signature.

★ And the precedent is **sixteen lines below it**, in the same file:

```clojure
;; count — the clojure surface name over the KEPT `length` primitive
(:wat::core::defalias :wat::core::count :wat::core::length)
```

So `wat/seq.wat` already establishes the pattern *"Clojure surface name over a kept primitive"* and
applies it to `length`/`count`. `foldl`/`reduce` is the same relationship — except the corpus went
the other way.

## ⛔ THE DEFECT: THE CORPUS PREFERS THE PRIMITIVE 15:1

```
572  calls to :wat::core::foldl     the primitive
 38  calls to :wat::core::reduce    the Clojure surface
```

In a language whose adoption thesis is *"frontier LLMs already have Clojure in their weights"*
(`docs/INTENTIONS.md`), **the Clojure name is the one being bypassed.** An LLM writing wat reaches
for `reduce` and finds a thin wrapper that 93% of the corpus ignores.

That is also a straight violation of the first floor discipline — **one canonical path per task.**
Today there are two, and the more-used one is the less-idiomatic one.

## ★ WHY THE NAME NO LONGER EARNS ITS KEEP

`foldl` is only meaningful as half of `foldl`/`foldr` — the direction suffix distinguishes a pair.
**`foldr` was retired in arc 118.B6b.** With its twin gone, the `l` distinguishes nothing; it is a
suffix answering a question nobody can ask any more.

⚠ **And two references to the dead name survive**, both correctly framed as history rather than as
callers — `wat-scripts/fixes/rete-where-per-type-spelling.wat:93` (*"the pair that used to…"*) and
`wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat:4` (*"a `Redispatch` alias
pointing at a core verb… that no longer exists"*). Not defects; noted so a later reader does not
mistake them for live callers.

## The shape a corrective stone would take

**Not a rename of the native.** The native's arity is 3; `reduce`'s surface is 2-and-3. Renaming
`foldl` to `reduce` would collide with the defclause that already owns the name.

The honest shape is the one `count`/`length` already models:

1. **`reduce` is the only public name.** The defclause stays and keeps both arities.
2. **The native loses its public spelling** — it becomes the 3-arity clause's implementation, named
   so it reads as a primitive rather than a surface (the `-spec` oracle pair at `wat/seq.wat:277`
   shows the house already distinguishes "the thing" from "the name for it").
3. **572 corpus call sites migrate `foldl` → `reduce`** — a textbook **R21 wat-fix codemod**, one
   rule, dry-run and diffed on a `/tmp` copy first. This is exactly the migration class that
   tooling exists for.
4. `foldl-spec` / `foldl-spec-walk` (`wat/seq.wat:277`) and their differential test rename with it,
   or the stone states why they keep the old name.

★ **Sequencing note, because it bites:** `:wat::core::foldl` is one of the 44 names still live in
`intrinsic_meta` (`WORKLIST-the-44-unhomed.md`, category A — the W7 HOF family, parked behind the
`effectful_by_prefix` question). **Renaming it before homing it means homing a different name
later; homing it first means the rename migrates a registered verb.** Doing the rename FIRST is
probably cheaper — a corpus codemod against an unhomed verb touches no registration — but that is a
judgement for whoever draws it, not a conclusion here.

## What this is an instance of

A name that was correct when minted, whose *justification* was retired underneath it, and which
survived because nothing forced the question. Arc 118.B6b removed `foldr` correctly; nobody asked
what the surviving `l` was still distinguishing.

Same class as `from-holon`'s dead 3-arg hint, found the same day.
`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

⛔ Not drawn. Builder's ruling on whether, when, and before or after the HOF homing.
