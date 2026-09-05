# NOTE — `:wat::eval::walk` is the last LANGUAGE verb that declares a `HolonAST`

> ✅ **CLOSED 2026-09-04.** The language surface no longer declares `:wat::holon::HolonAST` anywhere.
> Re-measured after the sweep: builtin type declarations name only `:wat::holon::BundleResult` and
> `:wat::holon::Holons`; the one verb scheme is `:wat::holon::Reckoner/new-discrete`; the corpus
> files are `wat/holon.wat`, `holon/Sequential.wat`, `holon/Ngram.wat`, `wat/test.wat`'s
> `assert-coincident`, and `wat/cache.wat`'s `hologram-svc`. **Every one is VSA.** Arc 294's ruling —
> *"the HolonAST and co tooling must only be used for VSA/HDC things"* — is now TRUE, measured
> rather than asserted. See *THE CLOSURE* at the foot of this file.
>
> ⛔ **CORRECTED 2026-09-04, SAME DAY, TWICE. The title is FALSE as written and the census below
> that produced it was blind in two separate ways.** Both corrections are recorded at the foot of
> this file under *THE CENSUS WAS WRONG TWICE*. Read that section before trusting anything here.

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


---

## ⛔ THE CENSUS WAS WRONG TWICE — corrected 2026-09-04, same day

The census above asked **"which verb SCHEMES mention `:wat::holon::HolonAST`"** and answered 2. The
question that mattered was **"where does HolonAST appear in the DECLARED SURFACE"**, and the answer
is larger. Two blind spots, found by two different people, neither by re-reading the census.

### Blind spot 1 — TYPE DECLARATIONS were never censused

Found by **the builder**, reading a rider's scoping comment in a live diff: *"it looks like there
are some holon internals still lingering where they shouldn't be."* Re-measured, anchored to
`register_builtin(TypeDef::…)` in `src/types.rs`:

```
:wat::holon::BundleResult   VSA — legitimate
:wat::holon::Holons         VSA — legitimate
:wat::eval::WalkStep        ⛔ Skip { terminal: :wat::holon::HolonAST , acc: A }
:wat::eval::StepResult      ⛔ AND IT VIOLATES ITSELF:
       StepNext        { form: :wat::WatAST }          ← ALREADY MIGRATED
       StepTerminal    { … : :wat::holon::HolonAST }
       AlreadyTerminal { … : :wat::holon::HolonAST }
```

★★★ **`StepResult` carries the intent inside the enum that breaks it** — one variant faces
`:wat::WatAST`, two do not. The unfinished anneal of 294 R9, in miniature, in a builtin type.

### Blind spot 2 — `--include=*.wat` CANNOT SEE WAT EMBEDDED IN RUST

Found by **a rider, from a red**. The stone's DESIGN claimed *"nothing in the corpus consumes the
holon"* on a corpus grep restricted to `*.wat`. Two consumers live as wat source inside Rust string
literals in `src/runtime.rs`'s own test module — `walk_w1_chain_to_terminal` and
`walk_w3_skip_short_circuits` — and both called `(:wat::holon::from-holon terminal)` on element 0.
**A file-extension filter is a claim about where a language lives, and in this repo wat lives inside
Rust too.**

## What SHIPPED, and the seam it leaves

`STONE-eval-walk-faces-watast` (arc 255) landed **narrowed**: `walk`'s declared return is
`:wat::WatAST`, converted at both construction sites, proven lossless by a composition probe
(`wat-scripts/scratch-pad/255-eval-walk-composition-roundtrip.wat`). The two red tests were repaired
with `:wat::eval-ast!` — the WatAST-native equivalent of `from-holon` — with neither assertion
weakened.

⚠ **The seam, recorded in the code at the site:** a `Skip` visitor still constructs a `HolonAST`
that `eval_walk` immediately converts back. A round-trip through two representations for no gain —
not lossy, not a behaviour regression, but real.

## What REMAINS — sized, so the next stone is briefable

A rider measured this under a STOP trigger rather than carrying it silently: **~34 edit sites across
4 files**, and it splits in two:

```
(a) WalkStep::Skip.terminal → :wat::WatAST
    SMALL — 2 corpus files + eval_walk's Skip-arm, which SIMPLIFIES (the conversion disappears).
(b) StepResult::StepTerminal / ::AlreadyTerminal → :wat::WatAST
    THE REAL BODY — touches eval-step!'s foundational regression harness (13 call sites through one
    shared driver), and several tests pattern-match HolonAST::Bind / Bundle / Thermometer directly
    in RUST, which becomes a semantic rewrite of what each asserts, not a renamed accessor.
    ⛔ AND IT CARRIES A DESIGN QUESTION, not a retype: the shared driver's Err-arm packs its error as
    `(:wat::holon::leaf (:wat::core::struct-field e 1))` so success and failure share ONE
    HolonAST-typed return. There is no wat-level primitive to wrap an arbitrary runtime string as a
    WatAST leaf. That fork must be answered before (b) can be briefed.
```

Also outstanding: 8 golden `<HolonAST>` string literals in `src/runtime.rs` and 1 in
`tests/value/wat_arc221b_keyword_dispatcher_completeness.rs`. They describe types that have not
moved and are correct today; they become stale the moment (a) or (b) lands.

## Disposition

**(a) is briefable now. (b) is blocked on the Err-arm fork.** Neither is on arc 255's chain; the
registry campaign remains the active work. Recorded so that whoever takes them starts from a
measured population rather than a grep — and so that nobody repeats either blind spot.


---

## ✅ THE CLOSURE — 2026-09-04

Four stones, in one session, after four wrong censuses:

```
STONE-eval-walk-faces-watast          walk's return                 :wat::WatAST   (narrowed)
FIX(109) threading macros             core.wat's -> and ->> lambdas  :wat::WatAST   (4 lines)
STONE-the-eval-surface-faces-watast   WalkStep::Skip.terminal
                                      StepResult::StepTerminal
                                      StepResult::AlreadyTerminal    :wat::WatAST
```

**The instrument that finally worked was the compiler**, on the builder's instruction — *"strike the
heresy where they stand; the compiler identifies the heretics immediately."*

⚠ And note WHICH compiler. Retyping the three fields produced **zero rustc errors**: `src/types.rs`
declares wat types as DATA (`TypeExpr::Path` strings), so the Rust compiler is structurally blind to
this entire class. **wat's own checker found them at startup**, one located message per site —
`:wat::eval::WalkStep::Skip: parameter #1 expects :wat::WatAST; got :wat::holon::HolonAST` — and the
worklist was 17 failures, sixteen through one shared driver. That is the census four greps could not
produce.

### Two things the sweep found that no census would have

- **A "design fork" that was not one.** A prior rider reported the shared driver's `Err` arm packs
  `(:wat::holon::leaf …)` so success and failure share one HolonAST return, and that no wat-level
  primitive wraps a runtime string as a WatAST leaf. Measured: `(:wat::holon::to-wat
  (:wat::holon::leaf x))` does, is registered, and satisfies a `:wat::WatAST` parameter. ⚠ It is a
  WART — building a holon to convert it immediately — and the follow-up is a `:wat::core::`-native
  WatAST leaf constructor. Named, not minted.
- **★ A TEST HARNESS THAT SKIPS THE CHECKER.** `step_holon_constructor_bundle` passed only because
  the pre-sweep `step_value_to_enum` still produced `Value::holon__HolonAST`, matching what the test
  expected while contradicting `types.rs`'s declaration. It uses `run_with_ctx`, which — unlike
  `run` — **never calls `check_program`**. Once the producer was corrected, that test would have
  begun failing with no type-checker able to see why. **A harness that skips the checker is a place
  where a declared type and a produced value can disagree indefinitely.** That is a general hazard,
  not a holon one, and it is unruled.

### Still open, and small

Nine golden `<HolonAST>` string literals were found and updated (8 in `src/runtime.rs`, 1 in
`tests/value/wat_arc221b_keyword_dispatcher_completeness.rs`) — the count the brief predicted,
exactly. The `Skip`-arm seam noted above is GONE: `WalkStep::Skip.terminal` is now `:wat::WatAST`, so
`eval_walk` matches `Value::wat__WatAST` directly and the round-trip conversion disappeared.
