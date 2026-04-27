# wat-rs arc 068 — Incremental evaluator (`:wat::eval-step!`) — INSCRIPTION

**Status:** shipped 2026-04-26. Three phases, three commits, ~6h
of focused work — substrate primitive for the form-as-coordinate
walk between Story 1 (the form's identity) and Story 2 (its
terminal value).

Builder direction (2026-04-26, mid-proof-016 review of an early
draft that used a custom `:exp::Expr` enum instead of real wat
forms):

> "the thing here... is that (x y z a b) for the coordinates to
> some 'is computed?' and 'has value?' would be forms of an wat
> program... `(let* ((something :i64) 42) (* something something))`
> would be a form we can use as an AST..."

> "we need an incremental eval here... when invoked....
> `(incremental-eval (form) (something form)) -> next-form`..."

> "everyone knows their immediate next form - they can (and should)
> put in the cache.. some adjacent caller can ask 'what is the next
> form for this-form?' and if they get a hit... they can query..
> 'what is the terminal value for this-form?'..."

The substrate had `eval-ast!` (full evaluation in one shot) but no
primitive that performs **exactly one reduction step**. Without it,
any consumer wanting to build a dual-LRU coordinate cache had to
re-implement wat's evaluator in user code — which can't happen
cleanly because HolonAST and WatAST are opaque at the user level
(no `head-of`, `args-of`, `bind?`, etc. destructors). This arc
ships the missing primitive.

---

## What shipped

| Phase | Commit | Module | LOC | Tests | Status |
|-------|--------|--------|-----|-------|--------|
| 1 | af43861 | `src/runtime.rs` + `src/check.rs` + `src/types.rs` — `:wat::eval-step!` plumbing: `StepValue` internal enum, `:wat::eval::StepResult` user-facing enum (Tagged variants `StepNext { form: :wat::WatAST }` / `StepTerminal { value: :wat::holon::HolonAST }`), `eval_form_step` entry mirrored on arc 066's `wrap_as_eval_result` shape, `RuntimeError::EffectfulInStep` + `NoStepRule` variants with kind tags `"effectful-in-step"` / `"no-step-rule"`. Phase 1 step rules cover literals only. | ~256 Rust + ~30 doc | 7 (literal-i64/bool/string/keyword terminal, NoStepRule fallthrough, arity mismatch, non-WatAST arg type-mismatch) | shipped |
| 2 | adaa3f6 | `src/runtime.rs` — per-form-shape step rules for arithmetic (poly + i64 + f64 +-*/, comparison, logical), control flow (`if` 5-arg shape; cond canonical → project to branch, else descend), `let*` (peel one binding per step + textual substitute name → rhs in remaining bindings + body), `match` (WatAST-level pattern matcher mirroring `eval_match`'s dispatch on Some / Ok / Err / keyword variant / bare binder / wildcard / literal), user function call (head = registered keyword path; descend args left-to-right; β-reduce by substituting params for arg forms in body; closure-bearing functions refuse with NoStepRule). Effectful prefix detection (`:wat::kernel::*`, `:wat::io::*`, `:wat::eval-*`, `:wat::load*`, `:wat::config::*`). `substitute` / `substitute_many` capture-free textual substitution per Identifier (name + scope-set) equality. `try_match_pattern_ast` WatAST-vs-WatAST pattern matcher. `step_to_watast` / `step_descend_then_fire` the descend-rule shared by every pure op. | ~742 Rust + ~30 doc | 13 new (arithmetic single redex / left descent / right descent, let* substitute / peel-first, if branch true / false / cond reduces, match canonical / scrutinee reduces, user function call, effectful kernel rejected, round-trip agrees with eval-ast!) | shipped |
| 3 | (this commit) | `src/runtime.rs` — holon constructors (`Atom` / `leaf` / `Bind` / `Bundle` / `Permute` / `Thermometer` / `Blend`) added via `step_holon_descend_then_fire` with a holon-canonical fire condition: a list whose head is itself a holon constructor (or `(vec :T <holons>...)` for Bundle's input shape) with recursively-canonical fields counts as a single value for the parent. The whole holon tree fires in one step instead of piecemeal — lifting a typed leaf back through a primitive WatAST would lose the HolonAST distinction the next constructor's `require_holon` check expects. Bundle's `Result<HolonAST, CapacityExceeded>` wrap peeled inline so step's terminal is uniformly a HolonAST. Bare `(:wat::core::lambda ...)` form steps to `Terminal(HolonAST::Atom(<canonical-form>))` per Q6. Pre-existing rot fix: poly-arith and strict-i64-arith both used bare `+/-/*/` which panicked in debug mode on simhash-derived i64 inputs (`tests/wat_simhash.rs::simhash_result_works_in_arithmetic`); switched to `wrapping_*` per Lisp/Scheme tradition. USER-GUIDE row + Story-3 worked example. | ~80 Rust + ~60 doc | 6 new (TCO under bound, holon Atom / Bind / Bundle / Thermometer, outer-form span survives rewrite) | shipped |

**wat-rs unit-test count: 681 → 707. +26. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo clippy --lib`: zero
warnings. `cargo test --workspace`: 0 failures (per arc 057's
`default-members`).

---

## Architecture notes

### Plotkin small-step on real wat forms — substitution by (name, scope-set)

The DESIGN's Q4 settled on textual substitution, not α-renaming.
Wat's hygiene model (arc 039 / Racket's sets-of-scopes) carries
scope sets on bare identifiers; `Identifier::PartialEq` is
`(name == name) && (scopes == scopes)`. Two distinct bindings of
the same name carry distinct scope sets and never alias under
substitution. The type checker enforces unique resolution per
binding site upstream, so capture cannot occur. The implementation
is a one-screen recursion on `WatAST`:

```rust
fn substitute(form, target, replacement) -> WatAST {
    match form {
        Symbol(ident, _) if ident == target => replacement.clone(),
        List(items, span) => List(items.map(substitute(...)).collect(), span),
        other => other.clone(),
    }
}
```

That's the whole substitution machine. Multi-binding is fold; match
arm rewrite is `substitute_many` over a `Vec<(Identifier, WatAST)>`
the matcher returns. No α-renaming, no fresh-variable generator,
no closure capture analysis at this layer.

### Two notions of canonicity

For arithmetic / comparison / logical (`step_descend_then_fire`),
canonical means **primitive literal**. Args fire when all are
IntLit/FloatLit/BoolLit/StringLit/Keyword. Lists are non-canonical
and descend.

For holon constructors (`step_holon_descend_then_fire`), canonical
means **primitive literal OR a recursively-canonical holon
constructor list**. `(Bind (Atom "k") (Atom "v"))` fires as a single
step because both args are themselves holon-canonical. The whole
holon tree collapses to one Terminal.

This is a deliberate deviation from strict Plotkin (single redex
per step). The motivation is type-loss: lifting a typed
`HolonAST::String("k")` back to a primitive `WatAST::StringLit("k")`
via `holon_to_watast` is structurally identical to a bare string
literal; the next `Bind` step's `require_holon` check fails because
the bare string isn't a HolonAST. Treating the entire constructor
tree as one canonical value avoids the round-trip and matches the
algebra's "the tree IS the value" framing.

For the consumer's cache: each form coordinate still gets its own
cache entry; the difference is only that a deep holon tree's
intermediate constructors don't get their own step. A consumer who
wants `(Atom "k")`'s cache entry would step that form on its own.

### Effectful prefix as a substrate boundary

`:wat::kernel::*`, `:wat::io::*`, `:wat::eval-*`, `:wat::load*`,
`:wat::config::*` refuse with `EvalError(kind="effectful-in-step")`.
The consumer pattern is "step until you hit one, fall back to
`eval-ast!` for that sub-form, resume stepping from the result."
The classification is prefix-based — adding new effectful
namespaces extends the list, not the algebra.

### Bundle's Result-wrap peeled inline

Bundle's signature is `:Result<HolonAST, CapacityExceeded>` — a
wat-side Result orthogonal to the `Result<StepResult, EvalError>`
that step's caller sees. Without peeling, step would Terminal-wrap
a `Value::Result` which has no HolonAST representation and
TypeMismatch. The fix is one match arm in
`step_holon_descend_then_fire` that unwraps `Value::Result` after
eval: `Ok(inner) → continue with inner; Err(struct) → propagate as
TypeMismatch so wrap_as_eval_result surfaces the capacity overflow
as the outer EvalError`. Q9 of DESIGN said capacity errors should
propagate as EvalError; this is how.

### Bare lambda terminal — Q6 made operational

Per arc 068 DESIGN Q6, a `(:wat::core::lambda ...)` step rule
returns `StepTerminal(HolonAST::Atom(<canonical-form>))`. The wat
form is lowered structurally via `watast_to_holon` and wrapped as
an opaque-identity `Atom` so cosine / hash / cache keys see the
lambda as a single coordinate. Closure-bearing lambdas (which
would have already produced a `Function` value with `closed_env =
Some` rather than reaching step mode as a literal `(lambda ...)`
form) aren't a step concern at this layer — the `step_user_call`
rule refuses functions whose `closed_env.is_some()` separately.

### Pre-existing i64 overflow rot

`tests/wat_simhash.rs::simhash_result_works_in_arithmetic` was
failing in debug builds because `eval_poly_arith` and
`eval_i64_arith` both used bare `x + y`, which panics on overflow.
SimHash returns near-i64::MIN values that overflow when doubled.
The test was authored at arc 051 (2026-04-?? in main); the rot's
provenance traces back to arc 050 where the poly arith first
shipped. Phase 3 fixes it by switching i64 +/-/*/ to wrapping
ops. Matches Lisp/Scheme tradition where machine-integer `+` is
the wrap-on-overflow shape; callers wanting checked arithmetic
compose explicit guards.

The visibility gap was that the test only fails in debug mode
(release builds wrap silently). `cargo test --workspace` in debug
flagged it on arc 068's branch; the fix lands in this arc's
commit per the no-pre-existing-excuse discipline.

---

## What this unblocks

- **proof 016 v4 — the expansion-chain proof.** A real wat form
  walked rewrite-by-rewrite, each form's coordinate written into
  a form→next LRU and (on terminal) a form→terminal-value LRU.
  Multi-walker cooperation falls out: any walker hitting a
  cached form shortcuts to the cached terminal.
- **BOOK Chapter 59's dual-LRU coordinate cache.** The chapter
  named the structure but had no substrate primitive to build on.
  This arc IS the primitive.
- **A future stdlib `wat/std/eval/cache.wat`.** With `eval-step!`
  in hand, the cache library can ship as user-level wat: take a
  cache + form, step + memoize. No new substrate work needed.

---

## What this arc deliberately did NOT add

- **Macroexpand-stepping.** Future arc when a consumer surfaces a
  need to step macro expansion incrementally.
- **Effectful stepping (`StepEffect` variant).** Future arc when a
  consumer needs to step an effectful op without the eval-ast!
  fallback. v1 refuses effectful ops cleanly so the boundary is
  observable.
- **Step-budget primitive (`step-bounded`).** Lab-userland helper,
  not substrate. A consumer that wants "step at most N times" can
  count in their wat-side driver.
- **Reckoner integration.** Downstream lab work (BOOK Chapter 55).
- **Closure-bearing lambda terminal.** Q6 settled on bare lambdas
  only. Functions with `closed_env = Some` (lambda values that
  captured outer bindings) refuse with NoStepRule. Future arc
  when a consumer needs higher-order programs that close over
  outer state.
- **`from-watast` step rule.** `(:wat::core::quote ...)` produces
  a `Value::wat__WatAST` which has no HolonAST representation
  through `value_to_holon`; needs a special-cased step rule. v1
  routes both quote and from-watast to NoStepRule for the
  consumer's eval-ast! fallback. Future arc.

---

## The thread

- **Arc 003** — TCO trampoline. The eval-substrate's mechanism
  for unbounded tail recursion. Step mode reuses the substitution
  semantics; tail calls become rewrites without stack growth.
- **Arc 028** — `eval-ast!` shipped with full one-shot evaluation.
- **Arc 057** — typed HolonAST leaves; algebra closed under itself.
  Made the WatAST → HolonAST lowering uniform (the basis for both
  `to-watast` and step's lift-back-to-WatAST).
- **Arc 066** — `eval-ast!` returns wrapped HolonAST per scheme;
  the substrate's promise becomes literal. Step mode mirrors this
  shape: `Ok(StepResult)` / `Err(EvalError)`.
- **2026-04-26 (DESIGN)** — proofs-lane drafts arc 068; Q1-Q9
  resolved.
- **2026-04-26 (Phase 1)** — plumbing + literal step rules ship
  (commit af43861).
- **2026-04-26 (Phase 2)** — arithmetic / control flow / let* /
  match / β-reduction step rules ship (commit adaa3f6).
- **2026-04-26 (Phase 3)** — holon constructors + lambda + USER-
  GUIDE + INSCRIPTION + i64-overflow rot fix ship (this commit).
- **Next** — proof 016 v4 rewrites against `:wat::eval-step!`.
  Dual-LRU cache library as user-level wat.

PERSEVERARE.
