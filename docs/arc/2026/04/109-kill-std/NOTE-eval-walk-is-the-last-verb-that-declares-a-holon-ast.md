# NOTE — `:wat::eval::walk` is the last LANGUAGE verb that declares a `HolonAST`

> **Found 2026-09-04 by the builder, from raw probe output**, mid-way through an arc 255 stone:
> a CEK-stepper probe printed `[#wat/holon 5 2]` and he asked *"wtf is a holonic data value doing
> here?... i thought we killed those for everything but vsa/hdc things..... we bootstrapped wat on
> holon's ast as holon is another representation for edn."*
>
> Recorded here per the arc-109 `NOTE-*.md` convention. **Nothing is fixed by this note.**

## The finding, measured

```
:wat::eval::walk
  IN   :wat::WatAST                            the form to walk
  IN   A                                       the accumulator seed
  IN   fn(A, :wat::WatAST, StepResult) -> …    the per-step callback ALSO speaks :wat::WatAST
  OUT  Result[ Tuple[ :wat::holon::HolonAST , A ], EvalError ]
                      ^^^^^^^^^^^^^^^^^^^^^^^
```

`src/check.rs`, `register_builtins`, `:wat::eval::walk`'s `TypeScheme`. Built to match by
`eval_walk` (`src/runtime.rs:12067`): `Value::Tuple([Value::holon__HolonAST(terminal), acc])`.

**It speaks `:wat::WatAST` on the way in and at every step, then hands the terminal form back as a
`:wat::holon::HolonAST`.** A caller that walks a wat form receives a holon.

## The census — what 109 killed, what legitimately survived, and what leaked

Anchored to the registration site, calibrated on a known case before the count was trusted:

```
SCHEMES in `register_builtins` mentioning :wat::holon::HolonAST ......... 2
   :wat::holon::Reckoner/new-discrete   ← VSA/HDC. Legitimate; this is what holon is FOR.
   :wat::eval::walk                     ← ⛔ THE LEAK. A language/eval verb, not a VSA one.
```

Everything else is one of two clean shapes:

- **VSA surface** — `:wat::holon::Atom` · `Bind/left` · `Bind/right` · `Bundle/children` ·
  `Bundle/first` · `Record` · `leaf` · `to-holon` · `HolonAST` itself. These are the domain.
  `src/intrinsic/holon/*.rs` (51 producers) and `src/holon/*.rs` (25) are its implementation.
- **INTERNAL bootstrap residue, correctly hidden** — the four reflection verbs
  `:wat::runtime::body-of` · `lookup-define` · `signature-of-defn` · `signature-of-fn` **all
  declare `(:wat::core::Option :- [:wat::WatAST])`** at the surface. They store `HolonAST`
  internally (`Binding::SpecialForm { signature: HolonAST }`, `src/reflect/lookup.rs:121`) and
  convert at the boundary — `holon_to_watast`, 4 call sites in `src/reflect/verbs.rs`. **The
  declared type is honest; the plumbing is residue.** This is the bootstrap the builder describes:
  wat was built on holon's AST because holon is another representation of EDN.

★ So the kill was substantially done. **One verb declares it, and 27 `Value::holon__HolonAST(`
producers remain in `src/runtime.rs`** — the residue's own volume, which the reflection verbs prove
can be hidden behind a `WatAST` surface.

## Why it matters, stated without overclaiming

- **The asymmetry is the defect, not the type.** `HolonAST` is a legitimate value. A verb that
  accepts `:wat::WatAST` at two positions and returns `:wat::holon::HolonAST` at a third makes its
  caller convert, for a reason the signature does not explain.
- **The reflection four prove the fix has a shape.** They face `WatAST` and convert internally.
  Whatever `walk` should do, the precedent for doing it is four verbs away in the same tree.
- ⚠ **It may be deliberate.** `walk` is the CEK stepper's surface and the terminal value might be
  intentionally holon-encoded. This note does NOT rule it a defect — it records that the asymmetry
  is undocumented at the site and that nothing in `walk`'s doc explains it.

## How it was found, because the method is the lesson

The rider that surfaced it reported the probe's output as *"returning `[5 2]`"* — **dropping the
`#wat/holon` tag.** The orchestrator repeated that summary. The builder read the raw terminal output
and saw it in one glance.

★ A paraphrase of output is not output. `[#wat/holon 5 2]` and `[5 2]` differ by exactly the token
that mattered, and the only reason it was caught is that someone looked at the bytes.
`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]` is the same family: the instrument
that summarises is the instrument that hides.

⚠ And a second, smaller error in the same exchange: the orchestrator then misread `[#wat/holon 5 2]`
as `#wat/holon [5 2]` and spent a step claiming the tag was in the wrong position. It is not —
`[#wat/holon 5 2]` is correct EDN, a 2-vector whose first element is the tagged `#wat/holon 5`
(verified: a bare HolonAST renders `#wat/holon 5`; a `Tuple[HolonAST, i64]` renders
`[#wat/holon 5 2]`). **There is no rendering defect. The anomaly is the signature.**

## Disposition

**OPEN, unruled, and not scheduled.** The registry campaign (arc 255) is the active work and this is
not on its chain. Recorded so the next reader of `:wat::eval::walk` does not have to rediscover it,
and so that whoever finally sweeps the last of the bootstrap AST has the census above rather than a
grep.
