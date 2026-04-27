# Arc 068 — Incremental evaluator (`:wat::eval-step!`)

**Status:** shipped 2026-04-26 (3 phases). Pre-implementation reasoning artifact.
**Predecessors:** arc 003 (TCO trampoline), arc 057 (typed HolonAST
leaves — algebra closed under itself), arc 058 (`HashMap<HolonAST, V>`
at user level), arc 066 (`eval-ast!` returns wrapped HolonAST per
its scheme).
**Downstream consumer:** holon-lab-trading proof 016 (the expansion-
chain proof) — needs to walk a real wat form one rewrite at a time,
recording `(form → next-form)` and `(form → terminal-value)` in two
caches as it goes. Also: BOOK Chapter 59's *dual-LRU coordinate
cache* (named, not yet built; this arc is the substrate primitive
the chapter pointed at).

Builder direction (2026-04-26, mid-proof-016 review of an early
draft that used a custom `:exp::Expr` enum instead of real wat
forms):

> "the thing here... is that (x y z a b) for the coordinates to
> some 'is computed?' and 'has value?' would be forms of an wat
> program... `(let* ((something :i64) 42) (* something something))`
> would be a form we can use as an AST... your tooling here doesn't
> seem to use wat forms but something... else..."

> "we need an incremental eval here... when invoked....
> `(incremental-eval (form) (something form)) -> next-form` ... we
> can keep passing next-form into the incremental eval until it
> terminates.... the termination cascades up the call chain..."

> "everyone knows their immediate next form - they can (and should)
> put in the cache.. so some adjacent caller can ask 'what is the
> next form for this-form?' and if they get a hit... they can query..
> 'what is the terminal value for this-form?'.... maybe they get a
> hit maybe they don't..."

> "as they compute the pointer to the next for they update the first
> data store.... form-to-next-form and once a terminal value is
> reached.. form-to-terminal-value can be decalred... terminal value
> is not guaranteed to be known if next form is known... N things
> maybe exploring some outer form at the same time... if one is able
> to find the terminal answer before someone else - the next caller
> can use that terminal value to shortcut their form traversal..."

The recognition: the substrate has `eval-ast!` (full evaluation in
one shot) but no primitive that performs **exactly one reduction
step**. Without it, any consumer that wants to build the BOOK
Chapter 59 dual-LRU coordinate cache has to re-implement wat's
evaluator in user code — which can't happen cleanly because
HolonAST/WatAST are opaque at the user level (no `bind?`,
`bind-lhs`, `head-of`, `args-of` destructors).

This arc ships the primitive that closes that gap. After it lands,
the dual-LRU coordinate cache is ~30 lines of pure wat user code on
top of `eval-step!` + `HashMap<HolonAST, HolonAST>` (arc 058) — no
further substrate work required.

---

## Why this arc, why now

The substrate has been walking toward this for several arcs:

- **Arc 057** — closed the algebra under itself. `HolonAST` is now
  `Hash + Eq` derive-clean, so it can serve as a cache key.
- **Arc 058** — `HashMap<HolonAST, V>` at user level. The cache
  containers are real.
- **Arc 066** — `eval-ast!` returns `Result<HolonAST, EvalError>`
  honestly. The full-eval primitive's contract is closed.

What's left for the dual-LRU coordinate cache to ship is the
**stepwise** half of the eval picture. Today's eval-ast! atomizes
the whole reduction; you can't observe the intermediate forms. The
chapter-59 cache needs each intermediate form because each is its
own coordinate, its own potential cache hit, its own potential
short-circuit for a parallel walker.

### What proof 016 surfaced

Proof 016 (the expansion-chain proof) was iterated three times in
one session. Each iteration the consumer pushed back:

1. **v1** — synthetic atoms `(double 5)` / `(square 3)`, no
   evaluator. Builder: *"those arn't things that can be eval'd"*.
2. **v2** — small `:exp::Expr` recursive enum with a stepping
   evaluator. Builder: *"still feels shallow.... real lambdas...
   real work"*.
3. **v3** — bigger Expr enum, TCO recursion, let-bindings. Builder:
   *"your tooling here doesn't seem to use wat forms but something...
   else"*.

The pushback at each step was the same shape: the form should BE
wat, not a parallel mini-language the proof invents. The substrate
should provide stepping; the proof should *use* it. Without
`eval-step!`, the proof has no choice but to invent its own AST
type, because the wat AST is opaque at user level and the substrate
doesn't ship a stepping primitive.

This arc is what unblocks proof 016's v4 — and any future consumer
that wants to walk a wat form one rewrite at a time.

### Cross-references

- BOOK Chapter 59 — *42 IS an AST*. Names the dual-LRU coordinate
  cache; the substrate primitive that closes the loop hadn't been
  built yet. This arc is that primitive.
- BOOK Chapter 55 — *The Bridge*. Names the two oracles (cache vs
  reckoner) and the thinker reshape from "predict" to "express".
  Same conceptual frame; different layer.
- `holon-lab-trading/wat-tests-integ/experiment/020-fuzzy-cache/` —
  proof 016's pair file. Will rewrite once this arc lands.
- `wat-rs/src/runtime.rs::eval` — the existing full-eval recursion;
  this arc reuses its dispatch for the per-redex rewrites.
- `wat-rs/src/runtime.rs::eval_form_ast` — arc 066's wrapper;
  `eval_form_step` lands as its sibling.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::eval-ast!` (`WatAST → Result<HolonAST, EvalError>`, full eval) | shipped (arc 066) |
| `:wat::core::quote` (capture form as WatAST) | shipped |
| `:wat::holon::to-watast` / `from-watast` (round-trip HolonAST ↔ WatAST) | shipped |
| `HolonAST` with `Hash + Eq` derive | shipped (arc 057) |
| `HashMap<HolonAST, V>` at user level | shipped (arc 058) |
| `wrap_as_eval_result` (Result wrap shape) | shipped (arc 028) |
| `eval` dispatcher in `runtime.rs` | shipped — reused for per-redex rewrites |
| `Substitution` machinery for `let*` body | shipped — reused for substitution-based steps |
| Span preservation through WatAST | shipped (arc 016) |
| TCO trampoline | shipped (arc 003) — consumer's walker uses it |
| `RuntimeError::EvalError` family | shipped — extended here with one variant |

Eight pieces in place. The new surface attaches at one dispatcher
entry plus one new public function plus one new enum variant.

## What's missing (this arc)

| Op / change | What it does |
|----|----|
| `:wat::eval::StepResult` (new built-in enum) | Two variants: `(StepNext :wat::WatAST)` and `(StepTerminal :wat::holon::HolonAST)`. The result of one rewrite. |
| `:wat::eval-step!` (new primitive) | `:wat::WatAST → :Result<wat::eval::StepResult, wat::core::EvalError>`. Performs one CBV reduction at the leftmost-outermost redex. |
| `eval_form_step` in `runtime.rs` | The Rust impl. Mirrors `eval_form_ast`'s Result-wrap shape; descends to the active redex; performs ONE rewrite; returns `StepNext` or `StepTerminal`. |
| `EvalError::EffectfulInStep { op: String }` | New variant. Returned when a form whose evaluation requires effects (kernel sends, IO writes, channel ops) is fed to `eval-step!`. Caller's signal to fall back to `eval-ast!` for that sub-form. |
| `EvalError::NoStepRule { op: String }` | New variant. Returned when the form's head has no step rule yet (e.g., a future stdlib op the stepper hasn't been taught). Caller falls back to `eval-ast!`. |
| USER-GUIDE rows + a Chapter-59-shaped example | Doc surface for the new primitive + StepResult + the consumer pattern (the dual-LRU cache loop). |

Six pieces. Most of the cost is in the step rules — case-by-case
mapping of WatAST shapes to "this is the redex; here's the rewrite."

---

## The new surface

### `wat::eval::StepResult`

A built-in enum, sibling of `wat::core::EvalError`. Lives in the
`wat::eval` namespace because `eval-step!` does too.

```scheme
(:wat::core::enum :wat::eval::StepResult
  (StepNext     (form :wat::WatAST))             ;; one rewrite happened, here's the next form
  (StepTerminal (value :wat::holon::HolonAST)))  ;; form had no redex; this is the value
```

Two variants, both tagged. The consumer pattern-matches:

```scheme
(:wat::core::match step -> :i64
  ((Ok (:wat::eval::StepResult::StepNext next))
    (:my::loop next))
  ((Ok (:wat::eval::StepResult::StepTerminal h))
    (:wat::core::atom-value h))                  ;; arc 066: HolonAST::I64(n) → i64
  ((Err e)
    (:wat::core::panic! "eval-step! failed")))
```

### `:wat::eval-step!`

```
:wat::eval-step!  :  :wat::WatAST → :Result<wat::eval::StepResult, wat::core::EvalError>
```

**Semantics:**

- Input: a WatAST representing a form to step.
- Output: `Ok(StepNext next-form)` if one rewrite happened, where
  `next-form` is the rewritten outer form (still a WatAST,
  re-feedable to `eval-step!`).
- Output: `Ok(StepTerminal h)` if the form is already a value with
  a HolonAST representation. Same wrap as arc 066's `eval-ast!`
  result (primitives wrapped in their HolonAST::I64/F64/Bool/etc.
  variant; HolonAST inputs pass through).
- Output: `Err(EvalError)` for malformed forms, effectful ops in
  step mode, or forms whose head has no step rule yet.

**Strategy:** **CBV, leftmost-outermost redex.** Standard Scheme/CL
order. The stepper descends the WatAST tree depth-first
left-to-right; at each node, if all sub-forms are values, fire the
op and return the rewritten outer form; otherwise descend into the
first non-value sub-form and step IT, returning the outer form with
that sub-tree replaced.

One step per call. The rewrite happens at the **innermost active
redex**, not at every redex on a single call. The consumer drives
the loop.

### Step rules

Per-form-shape rewrite rules. Each takes the form as a WatAST,
produces either `StepNext WatAST` or `StepTerminal HolonAST` (or
descends into a sub-form and steps it).

| WatAST shape | Step rule |
|----|----|
| `IntLit n` | `StepTerminal HolonAST::I64(n)` |
| `FloatLit x` | `StepTerminal HolonAST::F64(x)` |
| `BoolLit b` | `StepTerminal HolonAST::Bool(b)` |
| `StringLit s` | `StepTerminal HolonAST::String(s)` |
| `Keyword k` (where k is a unit-variant or callable) | resolve like `eval` does, return `StepTerminal` of the resolved Value's HolonAST representation |
| `(:wat::core::+ a b)`, `-`, `*`, `/`, `%`, `=`, `<`, `>`, `<=`, `>=` | If both args are literal: fire the op, `StepTerminal` the result. If left is literal and right isn't: descend right. If left isn't: descend left. |
| `(:wat::core::and a b)`, `or`, `not` | Same descent pattern as arithmetic; fire when args are literal. Short-circuit for `and`/`or` is a single-step rewrite. |
| `(:wat::core::if cond then else)` | Step `cond` until literal. Once `cond` is `BoolLit true`, `StepNext then`; if `false`, `StepNext else`. (One step at a time — the consumer keeps stepping.) |
| `(:wat::core::let* ((name :T val) ...) body)` | If `val` reduces, descend; once `val` is literal, `StepNext` is `body` with `name` substituted. Multi-binding `let*` peels one binding per step (since later bindings can reference earlier ones). |
| `(:wat::core::match scrutinee arm1 arm2 ...)` | Step `scrutinee` until canonical. Once canonical, `StepNext` is the body of the first matching arm with pattern-bound vars substituted. |
| `(:fn-name args...)` (registered user define) | Step args left-to-right. Once all args are values, `StepNext` is the body of the function with params substituted. (One step = one β-reduction.) |
| `(:wat::core::lambda (params) body)` | `StepTerminal` — a lambda is a value (a function). Encode as HolonAST::Atom(quoted-form) per Chapter 59 Story 1. |
| `(:wat::core::tuple a b c ...)`, `vec`, `hash-map` | Step the first non-value element; fire (build the container) when all are values. Result is a HolonAST representation of the container if expressible; `Err(NotHolonExpressible)` otherwise. |
| `(:wat::holon::Atom x)`, `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend` | Step args; fire the constructor when all are values; `StepTerminal` the resulting HolonAST. |
| `(:wat::core::struct-new ...)` | Step args; fire the constructor when all are values. Struct values that have a HolonAST representation: `StepTerminal`. Otherwise: `Err(NotHolonExpressible)`. |
| `(:wat::kernel::send/recv/...)`, `:wat::io::...`, etc. | `Err(EffectfulInStep { op: "<name>" })`. Consumer falls back to `eval-ast!` for this sub-form. |
| `(:wat::eval-ast! form)` | `Err(EffectfulInStep)` — recursive eval is a single opaque call; the inner form's stepping is its own concern. Consumers that want to step *into* an `eval-ast!` should pass the inner form directly. |

The list is long but mechanical. Most rules are "step left-to-right
through args, fire when all args are values."

---

## Decisions to resolve

### Q1 — Input/output AST type

Three options:

- **(a)** WatAST in, WatAST out for `StepNext`, HolonAST out for
  `StepTerminal`.
- **(b)** WatAST in, WatAST out always. `StepTerminal` carries a
  WatAST that's a literal.
- **(c)** HolonAST in, HolonAST out always. Stepper goes through
  `to-watast` internally.

**Recommended: (a).** The consumer feeds a WatAST (from `quote` or
from `to-watast`); the stepper returns either the rewritten WatAST
(re-feedable) or the terminal HolonAST (cache-keyable). Matches arc
066's `eval-ast!` shape on the terminal side. The consumer wanting
HolonAST identity for caching converts via `from-watast` on the
WatAST in `StepNext`.

Why not (b): callers can't easily compute `HashMap<HolonAST,
HolonAST>` keys without a from-watast-or-from-step normalization.
We'd push that work onto every consumer.

Why not (c): WatAST round-trip is lossy on identifier scope (per
arc 066 docs). Every step would re-lower, costing the substrate
nothing but losing precision. WatAST is the right working
representation for "form being stepped."

### Q2 — Effectful ops

A form like `(:wat::kernel::send chan v)` produces a side effect.
Three options:

- **(a)** Run side-effect ops to completion within one step (eval-
  ast! them internally). The step result's terminal IS the side
  effect's return value.
- **(b)** Reject effectful ops with a typed error. Consumer falls
  back to `eval-ast!` for sub-forms that contain side effects.
- **(c)** Add a `StepEffect` variant to `StepResult` carrying the
  effect descriptor. Consumer dispatches on it.

**Recommended: (b) reject with `EvalError::EffectfulInStep`.** The
chapter-59 cache is for forms that terminate without changing the
world — the whole reason the cache works as memoization is that the
form IS its return value. Effectful ops shouldn't be in the cache
anyway; every call has new side effects. Rejecting them honestly is
simpler than option (a)'s implicit side-effect-during-step
surprise, and keeps StepResult's two-variant shape clean.

Future arc may add `StepEffect` if a consumer surfaces a need (e.g.,
debugger-style stepping through effectful programs). Today's
consumer is the cache, which doesn't.

### Q3 — Forms with no step rule

A form whose head is a registered user function but the function
body uses ops the stepper hasn't been taught yet. Two options:

- **(a)** Return `Err(NoStepRule)`. Consumer falls back to
  `eval-ast!`.
- **(b)** Internally fall back to `eval-ast!` and return the result
  as `StepTerminal`. The form takes one giant step.

**Recommended: (a) return error.** Honesty over convenience. The
caller knows when the substrate hits a step boundary and chooses
how to handle it. Option (b)'s opaque jump is the same
side-effect-during-step surprise that Q2 rejects.

The step-rule coverage in v1 covers the substantive language core:
arithmetic, logical, control flow (`if`, `match`), bindings (`let*`),
function call, holon constructors. Roughly 80% of the wat surface.
Future arcs add rules for the remaining 20% as consumers surface
them.

### Q4 — Substitution semantics

When `let*` reduces or a function call β-reduces, we substitute
parameter names with values in the body. Standard substitution-
based semantics (Plotkin's CBV calculus). Open question:
**capture-avoiding** or **textual**?

Wat is hygienic (every binding has unique resolution per the type
checker). For step rules, **textual substitution** is correct:
substitute name-token with literal-token throughout the body, no
α-renaming needed. The type checker guarantees no capture risk.

### Q5 — Match arm selection

`(:wat::core::match scrutinee (pat1 body1) (pat2 body2) ...)` —
once `scrutinee` is canonical, find the first matching arm. The
stepper needs the same exhaustiveness/narrowing logic the type
checker uses (arc 055 recursive patterns). Two options:

- **(a)** Reuse the existing match-evaluator (`eval_match` in
  `runtime.rs`). One step does the full arm selection +
  pattern-binding substitution.
- **(b)** Step pattern-matching itself (one binding per step).

**Recommended: (a) one-step arm selection.** Pattern-binding is
not a meaningful unit of computation to expose — the consumer wants
"the chosen arm's body, with bound variables substituted." Doing
that in one step matches Plotkin's small-step semantics for case.

### Q6 — Lambda values

`(:wat::core::lambda (params) body)` is a value (a function). It
has no HolonAST representation today (per arc 066's
`NotHolonExpressible` family — closures are not algebra terms).

Two options:

- **(a)** `StepTerminal` returns `HolonAST::Atom(<canonical-form>)`
  — opaque-identity wrap of the lambda's quoted form per Chapter
  59 Story 1. The lambda's identity is on the algebra grid; can't
  be re-applied by the substrate, but cache keys work.
- **(b)** `Err(NotHolonExpressible)` — same as `eval-ast!`.

**Recommended: (a) opaque-identity wrap.** A lambda IS a coordinate
(per Chapter 54 — programs as coordinates). Returning it as
`HolonAST::Atom(<form>)` makes cache keys work for higher-order
programs. The substrate can't *apply* the lambda from the HolonAST
form — to do that, the consumer would round-trip via `to-watast`
and call `eval-ast!` on `(form arg1 arg2 ...)`. But the cache hit
is what matters here.

### Q7 — Span preservation

Every WatAST node carries a `Span`. Step rewrites should preserve
the **outer form's** span (the redex location). When we substitute
a value into a body, the substituted value carries its own span;
the surrounding body retains its original span.

**Recommended:** preserve spans through rewrites, using the
existing `WatAST::List(items, span)` shape. Errors from
`eval-step!` carry the span of the form that failed.

### Q8 — Multi-binding `let*`

`(:wat::core::let* ((a :T1 val1) (b :T2 val2)) body)` — peel one
binding per step, or fire all bindings at once?

Standard Plotkin small-step: peel one. After step 1 with `val1`
canonical: `(:wat::core::let* ((b :T2 val2[a→val1])) body[a→val1])`.

**Recommended:** peel one. Each binding's value can be
non-trivial; the stepper exposes each as its own redex. Consumer
gets fine-grained cache hits.

### Q9 — Capacity-mode interaction

`(:wat::holon::Bundle ...)` returns `Result<HolonAST,
CapacityExceeded>` (per arc 045). If a step fires Bundle and it
overflows, `StepTerminal` carries... what?

**Recommended:** the substrate's `:error` capacity mode raises
`RuntimeError::CapacityExceeded`. Inside `eval-step!`, that
propagates as `Err(EvalError::CapacityExceeded { ... })`. Consumer
treats it like any other failure; the form doesn't step further.
This matches `eval-ast!`'s posture.

### Q10 — Tests

Inline in `src/runtime.rs::mod tests`. Cover:

- Literal forms return `StepTerminal` with the right HolonAST
  variant: `5` → `Ok(StepTerminal HolonAST::I64(5))`.
- Single-redex arithmetic: `(+ 2 2)` → `Ok(StepTerminal
  HolonAST::I64(4))`.
- Multi-step expansion: `(+ (+ 1 2) 3)` → `Ok(StepNext (+ 3 3))`,
  then `Ok(StepTerminal HolonAST::I64(6))`.
- Let-binding: `(let* ((x :i64 5)) (* x x))` →
  - step 1: val canonical (`5`), substitute → `Ok(StepNext (* 5 5))`
  - step 2: → `Ok(StepTerminal HolonAST::I64(25))`
- If-branch: `(if true 1 0)` → `Ok(StepNext 1)`; one more step →
  `Ok(StepTerminal HolonAST::I64(1))`.
- Match: `(match (Some 5) ((Some n) n) (:None 0))` reduces
  scrutinee (already canonical), selects first arm, substitutes →
  `Ok(StepNext 5)`; one more → `Ok(StepTerminal HolonAST::I64(5))`.
- User function call: define `(square (n :i64) -> :i64) (* n n)`;
  step `(square 3)` → `Ok(StepNext (* 3 3))`; one more →
  `Ok(StepTerminal HolonAST::I64(9))`.
- Tail recursion: define `(sum-to (n :i64) (acc :i64) -> :i64) ...`;
  step `(sum-to 3 0)` through ~10 steps to `Ok(StepTerminal
  HolonAST::I64(6))`. Verify span preservation through the rewrites.
- Holon constructor: `(:wat::holon::Bind (:wat::holon::Atom "k")
  (:wat::holon::Atom "v"))` → `Ok(StepTerminal HolonAST::Bind(...))`.
- Effectful op: `(:wat::kernel::send chan 1)` →
  `Err(EffectfulInStep { op: ":wat::kernel::send" })`.
- Round-trip: capture form → step to terminal → compare to
  `eval-ast!` result. They agree.

Roughly ten test cases. Each ~10-15 lines of test code.

---

## What ships

One slice. Pure substrate addition.

- **`Value::eval__StepResult(Arc<StepResult>)`** — Rust-side
  representation. Two variants: `StepNext(Arc<WatAST>)`,
  `StepTerminal(Arc<HolonAST>)`. Mirrors `Value::Option(...)` /
  `Value::Result(...)` shape.
- **`eval_form_step` function** in `runtime.rs` — the implementation.
  Calls into the existing `eval` dispatch for sub-form reduction
  but pauses at the leftmost-outermost redex and returns the
  rewritten outer form.
- **Step-rule cases** for the WatAST forms listed in the table
  above. Each case is a small function (~10-30 lines) that knows:
  - Are all sub-args canonical?
  - If yes: fire, return `StepTerminal` or `StepNext`.
  - If no: descend into the first non-canonical sub-arm, step it,
    return the outer form with that sub-arm replaced.
- **`:wat::eval-step!` keyword wired** in the dispatcher.
- **`:wat::eval::StepResult` enum** registered via the same
  unit-variant-types path as `Option` / `Result`.
- **`EvalError::EffectfulInStep { op: String }`** and
  `EvalError::NoStepRule { op: String }` variants added to the
  EvalError enum.
- **`docs/USER-GUIDE.md`** — `eval-step!` row in the appendix
  (mirrored after `eval-ast!`'s entry); a Chapter-59-shaped
  example showing the dual-LRU cache loop in pure wat (~30 lines).
- **Tests** inline in `src/runtime.rs::mod tests` per Q10.

Estimated effort: ~400 lines Rust (most of it step rules) +
~200 lines tests + ~80 lines docs. Single arc; one slice; one
commit. Pattern matches arcs 058–067.

---

## Open questions (deferred to future arcs)

- **Stepping into macroexpand.** A form `(my-macro ...)` could be
  step-expanded by macroexpand-1 first, then stepped. Today `eval-
  ast!` does macro expansion at frozen-world setup; `eval-step!`
  inherits that posture (forms are post-macroexpansion). A future
  arc could add a `StepMacroexpand` variant or a separate
  `:wat::eval-step-macro!` primitive.
- **Stepping with effects.** Q2 rejected effectful ops in v1. A
  future arc adds `StepEffect` if a consumer (e.g., a debugger)
  surfaces a need for effectful stepping. Out of scope for the
  cache consumer.
- **Step-budget primitive.** A `:wat::eval-step-bounded!` that
  takes a max-steps and runs to a terminal or budget exhaustion.
  Convenience layer; not substrate. Lab-userland helper.
- **The dual-LRU cache library itself.** Once `eval-step!` lands,
  `:wat::eval::ExpansionCache` (HashMap+HashMap+walker) is ~30
  lines of pure wat. Worth shipping as `wat/std/eval/` after a
  consumer (proof 016 v4) exercises it. Not this arc.
- **Cross-thread cache sharing.** Multiple walkers cooperating on
  the same outer form via a shared cache. The substrate's CSP
  primitives (queue/topic/mailbox) handle handoff; the cache's
  HashMap is per-thread (zero-Mutex). Multi-walker is a *consumer
  pattern*, not substrate work.
- **Reckoner integration.** Chapter 55 names the second oracle
  (semantic labels). Pairing the cache (does it terminate?) with
  the reckoner (what label?) is a downstream lab arc, not
  substrate.

---

## Slices

One slice. Single commit. Pattern matches arcs 058–067.

If during implementation the step-rule coverage grows large enough
to warrant splitting (e.g., "v1 ships arithmetic + let* + if + match
+ user-defines; v2 adds holon constructors; v3 adds tuples/vecs"),
that split lives in BACKLOG.md. The DESIGN's posture is one slice
unless the impl pushes back.

## Consumer follow-up

After this arc lands:

- **Proof 016 v4** rewrites against `eval-step!`. The form is real
  wat: `(let* ((something :i64) 42) (* something something))`.
  The cache is `HashMap<HolonAST, HolonAST>` × 2. The walker is the
  ~30-line loop in BOOK Chapter 59. The custom `:exp::Expr` enum
  goes away.
- **`wat/std/eval/cache.wat`** — the dual-LRU cache library —
  ships as a wat-stdlib add when the proof confirms the surface.
- **BOOK Chapter 60 (or wherever the substrate hits next)** — the
  chapter that names what shipped: the substrate stops being
  "compute one shot" and starts being "compute one step at a time,
  cache the chain, share across walkers."

The diagnostic loop closes: substrate gap found (proof 016 can't
build cleanly without it) → arc DESIGN → arc shipped → consumer
uses the now-honest API → proof 016 v4 demonstrates the
chapter-59 architecture end-to-end.

PERSEVERARE.
