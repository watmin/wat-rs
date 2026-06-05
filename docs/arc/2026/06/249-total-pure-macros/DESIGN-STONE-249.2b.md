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

## The reachability boundary — DEFAULT-DENY (the interpreter design, grounded 2026-06-04)

The (ii) framing above ("reuse the pure `eval_*` helpers") needs one correction the crawl forced:
the existing helpers **evaluate their args by re-entering `eval`** (`eval_plus` → `eval(arg)` → add).
So a fence must propagate *recursively through arg-evaluation* — reusing a helper naively re-admits
the full impure dispatch through the back door. The fence is therefore not per-arm; it is a property
of the evaluator itself.

**`macro_eval` = `eval` restricted by a DEFAULT-DENY allow-list of blessed pure-total heads.**
Precedent: `eval-ast!` is already a *restricted eval* in this codebase (it refuses mutation forms —
`eval_ast_bang_refuses_mutation_form`, runtime.rs:25718). `macro_eval` extends that proven pattern.

> **DEFAULT-DENY is the load-bearing decision.** The gate is an **allow-list** (a head not blessed is
> refused), NOT a deny-list (refuse `:wat::kernel::*`). With deny-list, a new effectful prim added
> later is *silently admitted* until someone remembers to deny it — that is the F5 class, recurring.
> With allow-list, a new prim is *automatically refused* until someone deliberately blesses it. The
> "forgot to deny a new effect" failure is eliminated **structurally**, not guarded. This is the
> failure-engineering move + "shockingly stable is the minimum," made concrete.

**The blessed set (the reachable interpreter):**
- form-iteration over finite arg-lists: `map`, `filter`, `foldl`, `foldr`
- form-structure: `first`, `rest`, `cons`, `empty?`, `count`
- form-construction: quasiquote `` ` ``/`~`/`~@` (carrying its hygiene/unquote walk)
- control: `if`, `cond`, `match` (on form shape), `let`
- **local lambdas** (`fn`) — applied *only* by the bounded combinators (`map`/`fold`), never
  self-recursively
- a small set of pure expand-time scalar/collection ops (for counts/comparisons + computed-unquote:
  `i64::+`, `=`, …) — blessed leaves

**Two refusal gates (both default-deny):**
1. **Prim dispatch entry** — a keyword head not in the blessed allow-list → `MacroErrorKind::ImpureInMacro`
   (name owed an intueri cast). Catches every `:wat::kernel::*` and every future effect.
2. **`apply_function`** (the totality vector) — refuse calling a **top-level user `defn`** in macro
   mode (unbounded recursion); permit only local lambdas applied by the bounded combinators. v2's
   type-level `pure total` effect later admits verified user helpers; v1 closes the door.

**Totality by construction:** the only iteration is bounded combinators over *finite* lists (the
macro's args); no reachable path recurses unboundedly (the user-`defn` door is shut at gate 2).
**Purity by construction:** the only reachable heads are the blessed pure leaves (gate 1, default-deny).
Neither is a flag that can be forgotten — both are the *shape of the allow-list*.

## Representation (grounded 2026-06-04 — the open question, resolved + de-risked)

The engine is **not a new evaluator** — it is a *fenced restriction of an existing, working
pipeline.* The homoiconic boundary already exists and the computed-unquote path already uses it:
`unquote_argument` (expand.rs:882) does **form → `runtime::eval` → Value → `value_to_watast`
(runtime.rs:10458) → form.** `eval_quote` (runtime.rs:10318) / `:wat::holon::Atom` are the AST→Value
side; `value_to_watast` is the Value→AST side.

So the engine is precisely: **that same pipeline with `runtime::eval` replaced by a pure-only
`macro_eval`** (the restricted dispatch). A combinator-program macro body is a form; it evaluates
under `macro_eval` (params bound, blessed heads only) to a Value; `value_to_watast` converts the
result back to the expansion form. This is the *minimal* delta — a fenced variant of code that
already ships and is tested — which is exactly why (ii) is tractable, not a from-scratch evaluator.

The F5 reroute is then a *one-line swap* at expand.rs:866/940: `runtime::eval` → `macro_eval`. The
new capability (combinator-program bodies) is the same swap applied to the whole body instead of
just `,(expr)` sites.

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

## The body model — ONE kind: every body is a program (four-questions verdict, 2026-06-04)

There is **no template-vs-program distinction** — that binary was a fiction (a quasiquote template
*is* a program: an expression that builds a form). The four-questions (Obvious/Simple/Honest/Good-UX
all YES for unify; both "implicit-detect" and "explicit-marker" fail Honest — they reify a
distinction Clojure doesn't have) settle it:

> **Every macro body is a combinator program evaluated by `macro_eval`; a bare quasiquote is the
> degenerate (and most common) program.**

So `expand_template`'s `"body must be a quasiquote template"` gate (expand.rs:443) is **deleted, not
relaxed** — replaced by "evaluate the body under `macro_eval` with the params bound; the resulting
form is the expansion." Quasiquote becomes **one combinator** (the form-builder, carrying its
existing hygiene/unquote walk — `walk_template` becomes the quasiquote-combinator's implementation
*inside* `macro_eval`) rather than the only body shape.

**Backward-compatible by construction:** every existing stdlib macro body is a lone quasiquote — a
valid program — so they all keep working untouched, zero migration. No marker, no mode, no
detection. Purity/totality hold uniformly because there is *one* fenced evaluator.

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
