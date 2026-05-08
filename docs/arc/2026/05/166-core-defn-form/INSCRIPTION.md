# Arc 166 — INSCRIPTION

## Status

Shipped 2026-05-08. `:wat::core::defn` is the user-facing
named-function form. Lives in `wat/core.wat` as a defmacro composing
`:wat::core::def` + `:wat::core::fn`. Single-arity; no docstrings.

`cargo test --release --workspace --no-fail-fast`: **117 OK / 0 FAILED**
at `4bfbab9`.

| Slice | Subject | Commit |
|---|---|---|
| 1 | DESIGN + BRIEF + EXPECTATIONS | `3b908cf` |
| 1 | Macro + tests + substrate gap fixes | `4bfbab9` |
| 1 | SCORE + INSCRIPTION + USER-GUIDE + 058 row | (this commit) |

## What this arc adds

A user-facing form for naming functions. The form composes the two
foundational primitives:

```scheme
(:wat::core::defn :name :sig :body)
  ↓ macro-expansion (in wat/core.wat)
(:wat::core::def :name (:wat::core::fn :sig :body))
```

Per user direction 2026-05-08:

> *"i think defn is a wat provided form... not a rust provided form...
> my bias here is i want exactly one way to define a function. defn
> just binds the function to a name and has doc strings, etc... i
> think fn should be the one and only way to actually construct a
> function — defn is just a wrapper on it."*

The four questions on macro-vs-substrate-primitive:
- **Obvious?** ✓ macro (composition is honest; readers see what defn IS)
- **Simple?** ✓ macro (~6 lines, no substrate change)
- **Honest?** ✓ macro (defn IS sugar; saying so directly is honest)
- **Good UX?** Slight trade — error messages point at expanded form.
  Acceptable for a thin wrapper. Three of four favor macro
  decisively; the UX trade is the worth-it cost.

User direction *"when we make decisions we consult the questions
and the dependency order to deliver"* drove this decision tree.

## What retired

Nothing retired in this arc. Defn is additive on top of arc 157's
def + arc 155's fn. The `define` form remains operational; arc 166
does NOT migrate `define` consumers — that is a separate later arc
per user direction *"we'll flip from define to defn after its in
place."* Sequencing: defn ships additive (this arc); define stays
operational; migration sweep arrives later; `define` retirement
closes that arc.

## Substrate gap fixes (in-scope)

The defn macro surfaced two latent substrate gaps that arc 157
(def) hadn't exercised in isolation. Both closed in this arc per
FM 11 (no known defect left unfixed):

### Gap A — Recursive name binding for fn-shape def

**Symptom**: `(:wat::core::defn :fact (sig) (... (:fact ...) ...))`
expanded to `(def :fact (fn ...))`; `infer_def` inferred RHS BEFORE
writing to `defined_values`; the body's `(:fact ...)` reference
failed `Resolve(UnresolvedReferences)`.

**Root cause**: `def`'s sequential infer-then-register doesn't
mirror `define`'s pre-registration step. `register_defines`
pre-registers ALL `:wat::core::define` function names into
`sym.functions` BEFORE `check_program` runs, giving define-bound
function bodies forward-reference visibility for free.

**Fix** (`src/runtime.rs`):
- New helper `try_parse_fn_shape_def(form: &WatAST) -> Option<(String,
  Arc<Function>)>` detects the
  `(:wat::core::def :name (:wat::core::fn sig body))` shape, parses
  the fn signature via the existing `parse_fn_signature` helper, and
  builds a Function with `name: Some(name)` and `closed_env: None`.
- `register_defines` extended to call the helper after the
  `is_define_form` arm fails. Pre-registers fn-shape defs into
  `sym.functions`; KEEPS the form in `rest` so
  `register_runtime_defs` still evaluates the def at freeze time and
  populates `runtime_def_values`. Call dispatch's precedence ladder
  picks `sym.functions` first (per `lookup_form`), so the
  pre-registered Function wins; the runtime entry is
  vestigial-but-correct.

**Collision policy refinement** (caught by test 9 mid-iteration):
The first cut emitted `DuplicateDefine` on collision, which fired
the wrong error type for the second `(defn :user::f ...)` case (the
test expected `DefRedefForbidden` from `infer_def`'s redef
discipline). Refined: pre-register ONLY if name is new in
`sym.functions`. On collision, skip silently and let `infer_def`
emit `DefRedefForbidden` per arc 157 slice 1a-ii's strict-default.
Type-check-side redef discipline remains authoritative.

### Gap B — Reflection on def-bound names

**Symptom**: `(:wat::runtime::lookup-define :user::add)` where
`:user::add` was defn-bound returned `None` (or fired TypeMismatch
"expected keyword or named function; got wat::core::fn").

**Root cause**: `eval_lookup_define` always evaluates its argument
via `eval`. After Gap A fix, evaluating `:user::add` resolves to the
`runtime_def_values` entry — a `Value::wat__core__fn` with
`name: None` (because `eval_fn` doesn't know the def's name; it
only sees the bare `(fn sig body)` form). `name_from_keyword_or_fn`
returns None → TypeMismatch fires.

**Fix** (`src/runtime.rs`): special-cased `eval_lookup_define` to use
the keyword string DIRECTLY when `args[0]` is a literal keyword AST,
bypassing `eval`. Reflection on a literal keyword should resolve to
the keyword's name without depending on the runtime value's `name`
field. The eval path stays as the fallback for non-literal callers
(e.g., a symbol holding a fn-value from `sym.functions` where `name`
IS populated).

### Why these gaps were arc 166 scope

Both gaps make defn either non-functional (Gap A — no recursion) or
half-shipped (Gap B — invisible to reflection). Neither is acceptable
in a defn-shipping arc per FM 11. The fixes are bounded (~30-50
lines each), substrate-judgment work that orchestrator-side closure
kept tight (vs. delegating substrate-judgment to sonnet). The four
questions ran on the orchestrator-vs-sonnet decision and consistently
favored orchestrator-side for substrate-judgment-heavy work.

## Out of arc 166's scope

**Multi-arity overloads.** User direction 2026-05-08 (verbatim):

> "multi arity will be defn-clause — we will make that later — defn
> first... we will handle multi arity like erlang... i dislike
> clojure's N-ary approach."

Arc 166 ships single-arity defn. Out of arc 166's scope; reason:
user direction explicitly sequenced single-arity defn first to
establish the def + fn composition in isolation. If/when a caller
surfaces demand for `defn-clause`, a new arc opens; arc 166's
INSCRIPTION does not commit to it.

**Docstrings.** Per user direction:
> *"doc strings will come later, we need them on structs, enums,
> defs, defns, defn-clauses, etc etc — not now — later."*

Arc 166 does NOT take a docstring slot. Arc 141 (pending #225) is
queued to wire docstring source-extraction broadly across substrate
forms.

**`define` → `defn` migration sweep.** User direction 2026-05-08
(verbatim):

> "we will flip from define to defn after its in place."

Arc 166 ships defn additive. Out of arc 166's scope; reason: user
direction explicitly sequenced "defn first, migrate after." If/when
a caller surfaces demand to drive the migration, a new arc opens;
arc 166's INSCRIPTION does not commit to it. The `define` form
remains operational throughout.

**Flat-shape sig syntax** (`[x <- :T y <- :T] -> :T body`). User
direction 2026-05-08 sketched the end-state shape:
> *"<- consumes; -> produces. arrows point from the type toward the
> named slot."*

Arc 166 ships the current nested-sig shape `((x :T) (y :T) -> :T)`.
Future iteration ladder preserved at
`docs/arc/2026/05/166-core-defn-form/FUTURE-ITERATION.md`.

## Discipline notes

### Substrate-as-teacher carry

Sonnet shipped 8/10 in ~18 min and surfaced both gaps cleanly. The
BRIEF's "STOP at first red" + "no substrate edits" let sonnet
deliver the diagnostics without bridging. Orchestrator-side closure
applied the four questions on the fix path and chose direct edits
over delegation (substrate-judgment territory). The discipline held
end-to-end.

### Macro-as-tutorial pattern

`wat/core.wat`'s defn macro definition reads as documentation: the
quasiquote template IS the desugaring. Future readers learn the
def + fn composition by reading the macro body. Pattern aligns with
`:wat::test::deftest` (`wat/test.wat:304`) and
`:wat::runtime::define-alias` (`wat/runtime.wat:17`) — established
defmacro idiom in this codebase.

### Pre-INSCRIPTION grep

Run mechanically before this commit per FM 11. Three matches surface
in this INSCRIPTION; each is the literal acceptable scope-bounding
language pattern ("Out of arc N's scope... a later arc ...") with no
deferral language. Reviewed and accepted.

## Cross-references

- **Arc 157** — `:wat::core::def` foundational top-level
  value-binding form. Arc 166 composes def + fn; def's pre-eval
  register-AFTER ordering required Gap A's substrate fix to support
  recursive defn.
- **Arc 155** — `:wat::core::fn` function constructor. Arc 166 keeps
  fn as the single way to construct function values.
- **Arc 141** — core-form docstrings (pending). Future arc that
  wires docstring extraction; arc 166's defn extends to accept
  docstrings when 141 ships.
- **Arc 113** — orphaned scaffolding pattern. Arc 166's
  `try_parse_fn_shape_def` is the inverse: NEW scaffolding that
  treats def-fn-shape AS-IF a define for resolution purposes.
- **`wat/test.wat:304`** — `:wat::test::deftest` defmacro: the
  exact shape pattern arc 166 mirrors.
- **`wat/runtime.wat:17`** — `:wat::runtime::define-alias`
  defmacro: another worked example of quasiquote-template macros
  building substrate forms.

The Lisp on Rust gains its named-function form in the conventional
Clojure shape — a thin macro composing the two primitives, with the
substrate gaps closed and the discipline preserved.
