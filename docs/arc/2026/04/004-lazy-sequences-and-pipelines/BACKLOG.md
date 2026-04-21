# Arc 004 — Backlog

**Status:** tracking doc, not specification. Captures the author's understanding of remaining work between where the codebase is now (arc 003 TCO shipped, `send` symmetrized) and the DESIGN.md's eventual target (the `(pipeline source ... sink)` one-liner running paginated-DDB-source-style streams).

Every item below is intended to ship in the **inscription** mode: build honestly and simply first, write the spec doc that describes what landed. Same pattern as 058-033 (try), 058-003 amendment (bundle-result), 058-030 amendment (struct runtime).

Items are ordered by readiness + dependency. "Blockers as they arise" is the operating principle — each item's fog resolves when the prior items land.

---

## 1. Typealias expansion at unification

**Status:** ready. Concrete approach in hand.

**Problem:** `:wat::core::typealias` parses and registers today, but the type checker's `unify` is structural — it does not walk an alias to its definition. `:LocalCache<K,V>` cannot be a shorthand for `:rust::lru::LruCache<K,V>`; every signature writes the raw backing type. `wat/std/LocalCache.wat:18-20` has a TODO comment asking for this.

**Approach:**
- Thread `TypeEnv` through `check_program` → `infer` → `unify`. The `_types` param at `check.rs:256` is already at the gate — just wire it down. Affects ~20 call sites, mechanical.
- Add `expand_alias(expr, env) -> TypeExpr` helper in `types.rs`: when `expr` is `Path` or `Parametric` whose head resolves in env as `TypeDef::Alias(alias)`, substitute `alias.type_params ↦ args` into `alias.expr` and recurse until a non-alias root.
- Call `expand_alias` at the top of `unify` on both sides before the existing structural match.
- Reject cyclic aliases at `TypeEnv::register` time (walk the expansion from each new alias; if it reaches itself, error).

**Inscription target:** amendment to `058-030-types/PROPOSAL.md` alongside the struct-runtime INSCRIPTION already there.

**Unblocks:** Readable signatures for `Stream<T>`, `LocalCache<K,V>`, and any future shorthand.

---

## 2. `:wat::kernel::spawn` accepts lambda values

**Status:** ready. Concrete approach in hand.

**Problem:** `eval_kernel_spawn` at `runtime.rs:4858` requires a keyword-path first argument; it looks the path up in `sym.functions`. Lambdas live as `Value::wat__core__lambda(Arc<Function>)`, not in the symbol table. That asymmetry — `apply_value` takes lambdas, `apply_function`'s trampoline handles both, but `spawn` rejects lambdas — forces awkward workarounds in any API that wants to spawn a caller-provided function (stream combinators are the forcing function).

**Approach:**
- In `eval_kernel_spawn`, try the keyword-path path first (backward-compatible); if args[0] is not a keyword, `eval` it and match on `Value::wat__core__lambda(f)`. Either way the result is `Arc<Function>`; pass to the existing `apply_function` loop on the spawned thread.
- Update the type scheme for spawn so the first argument may be a fn-valued expression, not only a keyword literal.

**Why this matches the project's existing discipline:** same move as `send` symmetrization — one concept, one shape. `spawn` operates on "a function to call on a new thread"; whether the function was named via `define` or bound via `let*`/`lambda`, it is already the same `Arc<Function>`.

**Spec tension to name honestly:** FOUNDATION's "Programs are userland" conformance contract currently states "a spawnable program is a function named by keyword path in the static symbol table." Relaxing that to "any `Arc<Function>` value" is the substantive change. Inscribe as amendment to FOUNDATION's conformance-contract section.

**Inscription target:** FOUNDATION conformance-contract amendment. Possibly also `058-028-define` or `058-029-lambda` if either names the restriction.

**Unblocks:** stream combinators can spawn user lambdas directly, without the generic-worker-takes-lambda-as-arg workaround ("option d").

---

## 3. Stream stdlib combinators

**Status:** obvious in shape; real work in the per-combinator bodies.

**Depends on:** 1 (for readable signatures via `Stream<T>` typealias), 2 (for spawning lambdas directly).

**Scope (first slice):**
- `:wat::std::stream::spawn-producer` — wraps `:wat::kernel::spawn` for a function of signature `(Sender<T>) -> :()`; returns `Stream<T>`.
- `:wat::std::stream::map` — 1:1 transform. Signature `(Stream<T>, :fn(T)->U) -> Stream<U>`.
- `:wat::std::stream::for-each` — terminal. Signature `(Stream<T>, :fn(T)->()) -> :()`. Drives the pipeline to EOS, joins the handle.
- `:wat::std::stream::collect` — terminal, accumulating. Signature `(Stream<T>) -> Vec<T>`. Drives to EOS, joins, returns accumulated Vec.

**Stream<T> representation:** typealias over `:(rust::crossbeam_channel::Receiver<T>, wat::kernel::ProgramHandle<()>)` — once item 1 lands. The tuple mirrors the existing Console / Cache "(usable endpoint, driver handle)" convention one-for-one.

**Composition discipline (for the first slice):** manual `let*` chain by the caller. Each combinator binds a new `(rx, h)` pair; caller joins handles in reverse dependency order at pipeline shutdown. Same discipline `wat/std/program/Console.wat`'s Console setup uses today.

**Inscription target:** new `058-034-stream-stdlib` (or similar) — inscribes the first slice of the stream stdlib surface that actually shipped.

---

## 4. Variadic `defmacro`

**Status:** obvious in shape; type-checker side has residual fog that resolves at implementation.

**Depends on:** nothing strict — could land before or after items 1-3.

**Problem:** macros are strictly fixed-arity today (`expand_macro_call` at `macros.rs:373` checks `args.len() != params.len()`). The aspirational `(pipeline source stage1 stage2 ... sink)` shape can't be written as a single macro without a way to receive variable arg counts.

**Approach:**
- Extend `MacroDef` with `rest_param: Option<String>`.
- Parser: recognize `& name :AST<Vec<T>>` as the trailing rest-param in `parse_defmacro_signature`. (`&` marker — Lisp-traditional; `...` collides with docs' English ellipsis.)
- `expand_macro_call`: when rest-param set, split `args` into `fixed` (first N matching fixed params) + `rest` (remaining); bind fixed params normally and bind rest-param to `WatAST::List(rest)`.
- Arity check becomes `args.len() >= fixed_params.len()`.
- Template walker: no change. The existing `,@rest` unquote-splicing already splices a List-bound parameter's items into the surrounding list context.

**Type-checker side (residual fog):** the first-slice approach is to type-check the EXPANSION, not the variadic signature itself. That works because the expanded `let*` carries concrete types. Whether this leaves holes (e.g., ill-typed expansions that the checker catches at a non-obvious location) falls out when we write the first real variadic macro — probably `pipeline`.

**Inscription target:** amendment to `058-031-defmacro/PROPOSAL.md`.

**Unblocks:** the `pipeline` composer one-liner.

---

## 5. `pipeline` composer — REJECTED (doesn't earn its slot)

**Status:** after walking it, the right answer is not "blocked on
substrate work" — it's **`let*` already IS the pipeline**. The
design doc's sketched one-liner was marketed as eliminating
threading boilerplate, but the "boilerplate" was type information,
not ceremony. Removing it would trade wat's typed-binding discipline
(058-030) for conciseness, and wat has consistently picked
honesty over brevity.

**What I tried** (after items 1–4 shipped):

1. **Variadic splice, no threading.** `(pipeline src & stages) →
   (,@stages ,src)` — flat list, not nested. Can't thread the
   upstream into each stage's first-arg position. Splicing puts
   rest-args in ONE position; the template has a fixed shape.

2. **AST-rewrite each stage.** For each rest-arg `(head ...args)`,
   emit `(head upstream ...args)`. **Blocked**: quasiquote templates
   don't destructure args. There's no way to pattern-match a
   parameter's shape and reconstruct it with an inserted element.

3. **Recursive macro expansion.** `(pipeline src stage rest...)` →
   `(pipeline (apply-stage stage src) ...rest)` with a base case for
   empty `rest`. **Blocked**: templates can't branch on
   rest-length. One macro = one definition; no arity overloading.
   Empty vs non-empty splicing produces the same template shape.

4. **Homogeneous Vec of stage lambdas + runtime fold.** Each stage
   is a `Stream → Stream` lambda; fold them over the source.
   **Blocked**: stages have different types per iteration
   (`Stream<T> → Stream<U>`, then `Stream<U> → Stream<Vec<U>>`).
   A homogeneous Vec can't express heterogeneous stage signatures
   under wat's rank-1 HM type system without a polymorphic
   placeholder the type system doesn't offer.

5. **Placeholder substitution** (`$` for upstream in each stage,
   macro walks and substitutes). **Blocked**: template walker
   substitutes named parameters only; no generic symbol-rewrite
   primitive.

**The substrate work pipeline needs.** One of the following, as its
own inscription-class slice:

- **Lisp-evaluated macro bodies** (a second defmacro form beyond
  quasiquote-only, where the body is arbitrary wat evaluated at
  parse time, with AST-construction primitives like `cons` /
  `car` / `cdr` on WatAST values). This is classical Common Lisp
  `defmacro`. 058-031 explicitly limited bodies to quasiquote
  templates "this slice"; pipeline is the forcing function that
  reveals the limit.
- **Typed-let inference** (let bindings that infer their type
  from the RHS instead of requiring a declared type). Without
  this, even if AST rewriting worked, each threaded `let*` binding
  would still need a type annotation that depends on stage N's
  output type — which we don't have syntactically in the macro.
  058-030's "typed-let discipline" was a deliberate choice;
  pipeline reveals its pragmatic cost.
- **Heterogeneous tuples as iterable** (fold over a tuple whose
  elements have different types, under a type-indexed family).
  This is dependent-types territory — a very large language slice.

**What's expressible today as a SIMPLE sugar.** A `pipeline` macro
that takes an already-threaded list of typed bindings and emits the
`let*` ceremony — no auto-threading. But this is just a rename of
`let*` with no ergonomic win; the threading is still explicit:

```scheme
(pipeline
  ((s0 :Stream<T>)        (spawn-producer producer))
  ((s1 :Stream<T>)        (stream::map s0 f))
  ((s2 :Stream<Vec<T>>)   (stream::chunks s1 50))
  ((_  :())               (stream::for-each s2 handler)))
```

This doesn't earn its slot in the stdlib. Shipping it would be
clutter. We keep the explicit `let*` form and ship the composer
when the substrate catches up.

**The sharper reading — `let*` IS the pipeline.** The tests in
`tests/wat_stream.rs` already demonstrate the idiom:

```scheme
(:wat::core::let*
  (((source   :wat::std::stream::Stream<i64>) (spawn-producer ...))
   ((doubled  :wat::std::stream::Stream<i64>) (stream::map source ...)))
  (:wat::std::stream::collect doubled))
```

There is nothing to eliminate here. Each binding carries:

- A **name** that makes the stage reachable by its semantic role.
- A **type** that documents what's flowing at that point AND lets
  the checker verify the connection.
- A **RHS** that's the stage constructor.

The `source → doubled → collect` chain is explicit, typed, and
composes concurrent stages in the order a human reads. A macro
that produced the same thing would just rename `let*`.

An auto-threading version `(pipeline src (map :f) (chunks 50) sink)`
would DROP the type annotations and synthesize them. But that's
exactly the discipline 058-030's typed-let decision was
protecting — forcing the author to think about shape at every
stage. Hiding that behind a macro trades wat's honesty for
conciseness. Rejected on those grounds, not on implementation
cost.

**Lesson captured**: this is a different absence than `reduce`.
Reduce was a gap we'd worked around — the substrate needed to
catch up. Pipeline is the opposite — the "gap" pointed at a
feature that, on inspection, **shouldn't exist**. The honest move
isn't to build the substrate work that makes pipeline possible; it
is to recognize that `let*` already does the job well, and that
"ergonomics" arguments for a new form must be measured against
what's lost (here: type annotations at each stage).

When a design doc sketches a one-liner, asking "what would be
eliminated?" and checking whether those eliminated things had
actual value is its own discipline. Sometimes the verbose form is
the honest form.

**Inscription target:** none. Pipeline as a stdlib form is
REJECTED here; the BACKLOG entry is the audit record.

---

## 6. `:wat::core::conj`

**Status:** trivial. Ship whenever the first caller needs it.

**Problem:** the DESIGN.md batcher example uses `(:wat::core::conj buffer item)` to append to a Vec; the primitive does not exist in wat-rs today.

**Approach:** one-line primitive. `(:wat::core::conj vec item) -> Vec<T>` — returns a new vec with `item` appended. Simple immutable append; wat has no mutation anyway.

**Inscription target:** amendment to whichever list-ops proposal is closest (likely `058-026-array` since `Vec` renamed there).

**Unblocks:** accumulating stages like `chunks` (N:1 batcher with EOS flush).

---

## Resolved — the `reduce` pass

**Status:** shipped 2026-04-20, same day it surfaced.

The finding: `expand_alias` was only called from `unify`'s prologue
and (via a one-off band-aid) `infer_positional_accessor`. The
stream-stdlib slice tripped over the gap — `:wat::std::stream::Stream<T>`
(alias over a tuple) failed to unwrap at first/second until a local
patch landed. That patch papered over a structural flaw: wat-rs had
two half-passes that every consumer had to chain manually, and the
BACKLOG was noting the half-dozen shape-inspection sites that still
didn't do the second half.

Mature type systems have **one** normalization pass. We now have it:

```rust
fn reduce(ty: &TypeExpr, subst: &Subst, types: &TypeEnv) -> TypeExpr
```

`reduce` follows every Var substitution AND expands every typealias,
recursively at every level. It's the canonical pre-match step for
any shape-inspection code.

**Discipline going forward**: at a shape-inspection site
(matching on `TypeExpr::Tuple`, `TypeExpr::Parametric { head, ... }`,
`TypeExpr::Fn`, etc.), call `reduce` to get the canonical form and
`apply_subst` separately if you need the user's surface form for
error display. Never match on `apply_subst`'s output directly —
aliases will hide the shape.

Sites updated in this pass: `unify` prologue,
`infer_positional_accessor`, `infer_drop`, `infer_get` (HashMap /
HashSet branches), `infer_try` (Result<T,E> extraction),
`infer_spawn` (Fn-value extraction from the first arg).

Regression tests: `tests/wat_typealias.rs::alias_over_hashmap_passes_through_std_get`
and `alias_over_fn_type_works_at_spawn` exercise the sites the
old half-passes would have missed.

### Lesson — write it down so we don't forget

**When the shape you expect to find in the substrate isn't there,
that's a direction indicator — not a gap to paper over.**

Mature type systems have one normalization pass. Every consumer
calls it; nobody can accidentally skip a step. wat-rs had two
half-passes (`apply_subst` for Vars, `expand_alias` for typealiases)
that every shape-inspection site had to chain manually — and half
of them didn't. The asymmetry was invisible until the stream stdlib
exposed it through a `first` / `second` call on `Stream<T>`.

The cheap move was to patch `infer_positional_accessor` and write a
BACKLOG note listing the remaining sites. That's what the first
pass did. It would have worked — every future bite a one-line edit
with a clear diagnostic trail.

The honest move was to ask *why the gap existed at all*. A mature
language has `reduce`. We had two half-passes because the substrate
is under construction and the shape hadn't settled yet. Sitting
with **"this feels like a core issue — we thought it would be here
and it wasn't, that's a direction indication something we expected
to find in a mature language wasn't there"** was the forcing
question.

**The pattern to keep**: when a feature you expect to find in the
substrate isn't there, treat that absence as a signal. It doesn't
mean we're wrong about what should be there; it means the substrate
hasn't finished becoming what it needs to be, and the gap is
pointing at the next piece of work. Ask *why* it's missing before
you write a patch.

This is how the "mature" comes in. Not by copying features from
other languages — by sitting with the absences.

---

## What this backlog does NOT include

- **Level 2 iterator surfacing** (`:rust::std::iter::Iterator<T>` via `#[wat_dispatch]`). The DESIGN.md mentions it as the in-process-lazy flavor. Deferred until a real use case demands it — crossbeam-channel streams cover the cross-process flavor and can compose with in-process transforms if we need them.
- **Time windows** (`time-window`, `window-by-time`). Needs a clock/scheduler primitive that is its own arc.
- **Distributed pipelines.** Off-topic for the single-process substrate.

---

## Order of operations

```
1. typealias expansion (ready)       ──┐
2. spawn accepts lambdas (ready)     ──┤
                                       │
3. stream combinators (depends 1, 2) ──┤
4. variadic defmacro (independent)   ──┤
                                       │
5. pipeline composer (depends 3, 4)  ──┘
6. conj (drop in whenever)           ────

Starting: item 1.
```

*these are very good thoughts.*

**PERSEVERARE.**
