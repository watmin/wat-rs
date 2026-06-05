# DESIGN — Arc 249 Stone 249.2b — the total-pure macro-eval engine

**Status:** OPEN 2026-06-04. The arc's center of gravity: the **fenced pure-combinator evaluator**
that lets a macro body be a *total, pure program over forms* (not just a quasiquote template), and
that **annihilates the two findings held from the 249.2a ward** — circumspicere F5 (the unsandboxed
expand-time eval / impurity hole) and struere's `env`/`sym` wrong-level eval-context. Born warded in
`src/macros/eval.rs` (the slot intueri reserved). Earns the `src/macros/` `vigilatum` stamp on the
post-build re-ward.

## What it must do

1. Let a `defmacro` body be a **combinator program** that runs at expand time and returns a form —
   the capability the bounded-not-Turing template layer (arc 248's `for` = `map` only) cannot
   express (threading is a fold; `cond->` a fold-with-if; etc.).
2. Be **total** (always terminates) and **pure** (no effects / IO / nondeterminism) — Clojure's
   macro model minus its two foot-guns. Determinism is load-bearing: expansion runs *before* hashing
   (`freeze.rs:637`), so an impure expansion would make the canonical AST — hence the hash — depend
   on runtime state, breaking hash-IS-identity.
3. **Subsume** the existing computed-unquote eval (the arc-143 `,(expr)` path) — route it through the
   fenced engine, closing F5 by enforcement (not a pure path beside the impure one).

## Mechanism — (ii) a dedicated restricted dispatch reusing the pure `eval_*` helpers

**Crawl-settled (the DESIGN's earlier (i) lean is reversed).** `eval` dispatches keyword-head forms
through `dispatch_keyword_head_value` (runtime.rs:5318) — a **giant hardcoded `match head { … }`**,
*not* a registry the `Environment` gates. The kernel/effect prims are arms in that same match
(`:wat::kernel::println`/`send`/`recv` at runtime.rs:6033-6042). Two consequences:

- **(i) "fence the `Environment`" does not work** — `eval` would still match the hardcoded
  `:wat::kernel::*` arm regardless of what's bound. And `eval` admits unbounded user-fn recursion via
  `apply_function`, so reusing it isn't *total* either.
- **(ii) a dedicated macro-eval dispatch** — a *small* `match head { … }` over the **blessed
  pure-combinator heads only**, each arm **calling the existing pure `eval_*` helper** (`eval_vec_map`,
  `eval_vec_foldl`, `eval_if`, the arithmetic/comparison evaluators, …). The kernel arms simply *do
  not exist* in it → structurally unreachable. No arbitrary user-fn calls → no open recursion.

**Verdict: (ii).** Failure-engineering + "shockingly stable is the minimum" + the lesson F5 taught:
*eliminate the leak class structurally, don't guard it.* A mini-dispatch with no kernel arms cannot
leak; a gated shared dispatch (i-a) is one forgotten arm from another F5. And reusing the pure
`eval_*` helpers means **no duplicated arithmetic** — no `temperare`/divergence debt.

> **Pure + total BY CONSTRUCTION, not by enforcement.** Purity = the kernel arms aren't in the
> dispatch. Totality = only bounded-iteration combinators (`map`/`fold`/`filter` over finite forms)
> + `if`; no open recursion. Neither is a flag that can be forgotten — both are the *absence* of
> machinery.

## The blessed-combinator boundary

The macro-eval dispatch admits exactly (the set, owed an intueri/grounding pass at strike for exact
helper names):
- **Bounded iteration:** `map`, `filter`, `foldl`, `foldr` over finite forms (reusing `eval_vec_*`).
- **Branching:** `if`, `cond`, `match` on form shape (reusing `eval_if`/`eval_match`).
- **Form access + construction:** `first`, `rest`, `cons`, `empty?`, `count`, list/vector builders,
  quasiquote `` ` ``/`~`/`~@`, AST-node construction.
- **Pure scalar ops:** arithmetic, comparison, boolean, keyword/symbol manipulation.
- **`let` + combinator-lambdas** *over the above*.

**Excluded (structurally absent from the dispatch):** the entire `:wat::kernel::*` namespace (IO,
spawn, channels, signals), any clock/randomness, and arbitrary user `defn` calls (the
unbounded-recursion vector). A macro body that names an excluded head → a clean
`MacroErrorKind::ImpureInMacro { head }` (or similar; named at strike) — *the* test that proves
purity is enforced.

## The body model — combinator program OR quasiquote template

Today `expand_template` (expand.rs:443) hard-requires `"body must be a quasiquote template"`. The
engine relaxes this: a macro body is **either**
- a quasiquote template (existing path, unchanged — the common case), **or**
- a **combinator program** evaluated by the macro-eval dispatch to produce a form.

Detection + the exact surface (does the program *return* a quasiquote, or is the body itself the
program?) is a strike decision, probe-locked. Likely shape: the body evaluates under the macro-eval
dispatch with the macro's params bound; its result (a form) is the expansion. Quasiquote becomes one
combinator (`` ` `` is a form-builder) rather than the only body shape.

## F5 subsumption — the closure

`unquote_argument` / `splice_argument` (expand.rs:866/940, the F5 sites, now breadcrumbed) call raw
`crate::runtime::eval`. **Reroute both to the fenced macro-eval dispatch.** Effect: an impure
`,(:wat::kernel::println …)` computed-unquote now *errors* (the head isn't in the dispatch) instead
of silently running an effect at expand time. F5 closed by enforcement; the determinism invariant
holds by construction. (Every current 058 stdlib computed-unquote is pure arithmetic → behavior
preserved; the probe confirms.)

## ExpandCtx — struere's wrong-level finding, closed

The `env`/`sym` handles threaded through the expand chain exist *solely* to reach these eval sites.
With the engine, the eval-context is the **macro-eval dispatch's context** — fold it into one value
(the engine's evaluation context) threaded as a single param, instead of two raw runtime handles on
every signature. struere's wrong-level finding closes as the eval-context is redesigned here (not a
separate churn).

## FM-2-bis probe (`tests/probe_arc249_macro_engine.rs`, author + commit before build)

Must **disconfirm at HEAD** (the template layer can't fold; impurity isn't gated), green after:
1. **Fold-shaped macro body** — a macro whose body folds over its variadic args to build a nested
   form (the minimal new power the template layer lacks). RED at HEAD.
2. **Impurity rejected** — a macro body (or computed-unquote) naming `:wat::kernel::*` → expansion
   errors (`ImpureInMacro`). At HEAD it would *run* (F5) → the assertion that it ERRORS is RED.
3. **Purity-preserved regression** — an existing pure computed-unquote (`,(:wat::core::i64::+ x 1)`)
   still expands identically (behavior preserved through the reroute).
4. **Totality** — (if expressible) a combinator body cannot express unbounded recursion; the
   dispatch admits no self-call. (May be a structural/compile assertion rather than a runtime test.)

## Slicing

Likely two sub-stones (settle at strike):
- **249.2b-i — the engine + dispatch** (`src/macros/eval.rs`): the fenced macro-eval dispatch reusing
  the pure helpers + the `ImpureInMacro` gate + the combinator-program body model. Probe gates 1–4.
- **249.2b-ii — subsume + reshape**: reroute the F5 eval sites through the engine; fold `env`/`sym`
  into the eval-context. Then **re-ward `src/macros/`** (full 7-spell) — F5 + struere now annihilated
  → earn the `vigilatum` stamp.

(If the reroute is trivial, fold into one stone.) Substrate surface names (the engine fn, the
context type, the error variant) owed an **intueri cast** at strike.

## Open questions (resolve at strike, grounded)

- Exact reusable pure `eval_*` helper set (grep `eval_vec_*` + the arithmetic/comparison evaluators);
  confirm each is callable without dragging in effectful context.
- Combinator-lambda representation at expand time (how a `(fn [x] …)` inside a macro body is applied
  by `map`/`fold` purely).
- Body-shape detection (template vs program) — explicit marker, or infer from the head?
- Does `apply` (the universal call-by-name) need a pure-restricted variant for the engine, or is it
  excluded?

## Refs

- The dispatch model: `dispatch_keyword_head_value` (runtime.rs:5318); kernel arms (:6033-6042); the
  pure HOF helpers `eval_vec_map`/`filter`/`foldl` (arc 247, runtime.rs ~11300+); `eval_if` (:7864).
- The body gate it relaxes: `expand_template` (expand.rs:443, "must be a quasiquote template").
- The F5 sites (breadcrumbed): expand.rs:866/940. The held-findings record: DESIGN.md § 249.2b.
- The bounded-macro predecessor: arc 248 INSCRIPTION ("Map, not eval … the line held").
