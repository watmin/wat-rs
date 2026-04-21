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

## 5. `pipeline` composer

**Status:** genuine design fog. Revisit after items 3 and 4 are in code.

**Depends on:** 3 (need real combinators to compose) + 4 (variadic).

**The fog:** the user's sketch `(pipeline source (stream::map :f) (stream::chunks 50) sink)` has each "stage" missing its upstream argument. The macro has to rewrite each stage form by prepending the threaded upstream. Doable in principle — each stage is a list `(head ...args)` and the macro emits `(head __prev ...args)` — but edge cases (stages that aren't in call form, inline lambdas, sinks that take the whole stream) have not been walked through.

Expect the shape to clarify from writing real pipelines the verbose way first (item 3 + manual `let*`), which reveals what the composer has to produce.

**Inscription target:** TBD. Likely a new `058-035-pipeline-composer` or an amendment to the stream stdlib inscription.

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
