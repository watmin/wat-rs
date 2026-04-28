# wat-rs arc 070 — `:wat::eval::walk` and `StepResult::AlreadyTerminal` — INSCRIPTION

**Status:** shipped 2026-04-27. Two phases, two commits, ~3h of
focused work — substrate primitive that lifts the walker pattern
proofs 015/016/017/018 each reimplemented, plus a third
`StepResult` variant that distinguishes "already a value" from
genuine `Err`.

Builder direction (2026-04-27, mid-proof-018 review after the
walker silently swallowed the case where eval-step! returned `Err`
on a trader-shape thought):

> "this realization you just had... we need this as a thing wat
> provides... the terminal-ness — what's missing from wat-rs for
> this?"

> "that sounds like the thing — that sounds /very/ core"

The substrate already KNEW which forms were value-shape and which
were genuine errors. It just didn't surface that distinction.
Walkers across four proofs each had to invent the discrimination
locally. This arc encodes it once.

---

## What shipped

| Phase | Commit | Module | LOC | Tests | Status |
|-------|--------|--------|-----|-------|--------|
| 1 | 97c827b | `src/types.rs` — `:wat::eval::StepResult` gains `AlreadyTerminal { value: HolonAST }` (third Tagged variant). `src/runtime.rs` — `StepValue::AlreadyTerminal` Rust-side variant + `step_value_to_enum` arm. `try_recognize_holon_value(form: &WatAST) -> Option<HolonAST>` recursive structural-walk: if every node is a holon-value shape, rebuilds the matching HolonAST and returns Some; otherwise None. Reduction-shape forms (arithmetic, comparison, special forms, user fn calls, source-form Bundle) return None. `step_form` checks this first; on Some, short-circuits to AlreadyTerminal. `Atom` recognition mirrors `value_to_atom`'s polymorphic dispatch (arc 057): primitive args → typed leaves, nested holon-ctor args → opaque-identity Atom wrap. Wat-side `step-to-terminal` test driver gains a third match arm. | ~270 Rust + ~3 wat | 30 step tests (3 new: `step_already_terminal_on_lifted_bundle`, `step_already_terminal_on_holon_constructor_call`, `step_terminal_on_arithmetic_redex`) | shipped |
| 2 | (this commit) | `src/types.rs` — `:wat::eval::WalkStep<A>` parametric enum (Continue / Skip variants). `src/runtime.rs` — `eval_walk` iterative loop calling `step_form` + `apply_function` per coordinate; visitor returns WalkStep<A>; Continue + StepNext recurses, Continue + terminal returns, Skip stops anywhere. Test setup gains `register_enum_methods` so synthesized variant constructors `:wat::eval::WalkStep::Continue` etc. resolve. `src/check.rs` — type scheme for `:wat::eval::walk` with `type_params: ["A"]`, full visitor signature, return tuple `(HolonAST, A)` wrapped in `Result<_, EvalError>`. `docs/USER-GUIDE.md` — Story 3 worked example rewritten around `walk`; appendix rows for both new types. | ~190 Rust + ~50 docs | 4 walk tests (`walk_w1_chain_to_terminal`, `walk_w2_already_terminal_input`, `walk_w3_skip_short_circuits`, `walk_w4_propagates_eval_step_err`) | shipped |

**wat-rs unit-test count: 723 → 730. +7. Workspace: 0 failing.**

Build: `cargo build --lib` clean. `cargo clippy --lib`: zero
warnings. `cargo test --workspace`: 1093 passed, 0 failed.

---

## Architecture notes

### `try_recognize_holon_value` mirrors `holon_to_watast` in reverse

The detection is structural: each WatAST shape that
`holon_to_watast` (arc 057) emits has a corresponding inverse in
`try_recognize_holon_value`. Primitives lift to typed leaves;
holon-constructor calls (`Atom`, `leaf`, `Bind`, `Permute`,
`Thermometer`, `Blend`) lift to their matching HolonAST nodes;
bare-list (Bundle-shape) lifts to `HolonAST::bundle(children)`. The
inverse fails on any reduction-shape (arithmetic, comparison,
special forms, user fn calls, source-form Bundle constructor —
those have work to do, not values).

`Atom` specifically mirrors `value_to_atom`'s polymorphic dispatch:
primitive args lift to typed leaves; nested holon-ctor args wrap as
opaque-identity `Atom`. The substrate's invariant — `Atom` never
wraps a primitive directly because primitives are already typed
leaves — propagates through the recognizer.

`leaf` (arc 065) is primitive-only by design; the recognizer
returns None for non-primitive args, matching eval-time behavior.

### Behavior changes from arc 068

- Primitive literals (`42`, `true`, `"hi"`, `:keyword`):
  StepTerminal → AlreadyTerminal.
- Holon constructor calls with all-canonical args
  (`(:wat::holon::Atom "k")`, `(:wat::holon::Bind ...)`,
  `(:wat::holon::Thermometer 0.5 0 1)`):
  StepTerminal → AlreadyTerminal.
- Bundle-lifted bare-list forms (the case proof 018 hit):
  Err(NoStepRule) → AlreadyTerminal.
- Source-form `(:wat::holon::Bundle (vec :T ...))`: still
  StepTerminal — that's a real constructor call (capacity check,
  encoder pipeline). The lifted Bundle (bare list) takes the
  AlreadyTerminal path because the bare-list shape isn't a function
  call.
- Arithmetic / comparison / logical / user-fn fires unchanged:
  StepTerminal still means "this step reduced a redex."

### `walk` is iterative, not recursive

Long chains shouldn't blow the Rust stack. The walker's loop
mutates `current_form` and `acc` in place, calling `step_form` +
`apply_function` per iteration. The wat-side visitor IS a function
call, but the walker's outer loop is a flat loop. Pathological
infinite chains would loop forever in walk — Q4 of DESIGN deferred
the max-depth parameter; the wat-rs runtime's existing recursion
caps inside `apply_function` provide the per-call ceiling.

### `WalkStep<A>` is a generic built-in enum — first one in wat-rs

Pre-arc-070, every built-in enum (`StepResult`, `ThreadDiedError`,
`Failure` etc.) was monomorphic. `WalkStep<A>` is the first
type-parameter-ed built-in. The substrate's existing `EnumDef.type_params`
field already supports this; arc 070 just exercises it. The check.rs
type scheme uses `TypeExpr::Path("A")` for the parameter (lexically-
scoped per the comment in `TypeExpr::Path`'s docstring), and
`TypeExpr::Parametric { head: "wat::eval::WalkStep", args: [...] }`
for the parameterized return type.

### `register_enum_methods` now runs in unit-test setup

Pre-arc-070, the unit-test SymbolTable only ran
`register_struct_methods` — synthesized struct accessors but no
tagged-variant constructors. Arc 070's tests need
`(:wat::eval::WalkStep::Continue acc)` and
`(:wat::eval::WalkStep::Skip terminal acc)` to resolve at the test
boundary. One-line addition to `stdlib_loaded()`.

### Visitor never sees Err

Q5 of DESIGN: `eval-step!` errors propagate as `walk`'s outer
`Result::Err` without invoking the visitor. The visitor's
contract is "I see every coordinate that succeeded a step." Errors
are out-of-band. If a consumer wants to recover, they wrap walk and
match on the Result — the substrate doesn't second-guess.

---

## What proof 018's walker collapses to

Before — ~50 lines of recursion, manual cache record on the way
down and the way up, silent-swallow Err branch:

```scheme
(:wat::core::define
  (:exp::walk-and-record (form-h :wat::holon::HolonAST) (tier :exp::L1Tier)
                         -> :(wat::holon::HolonAST,exp::L1Tier))
  (:wat::core::let*
    (((form :wat::WatAST) (:wat::holon::to-watast form-h)))
    (:wat::core::match (:wat::eval-step! form)
      ((Ok r)
        (:wat::core::match r
          ((:wat::eval::StepResult::StepTerminal t) ...)
          ((:wat::eval::StepResult::StepNext next) ... recurse ...)))
      ((Err _e) ... silent fallback ...))))
```

After — one call, one fold function. The visit-fn handles all
three step-result variants distinctly (no silent fallback). The
walker handles iteration. Backprop-on-the-way-up disappears
because cache lookup short-circuits via `Skip` — what backprop
was approximating.

---

## Test coverage

Phase 1 (3 new step tests):
- **`step_already_terminal_on_lifted_bundle`** — the proof-018
  failing case made positive: bare-list Bundle lift returns
  AlreadyTerminal with the rebuilt HolonAST.
- **`step_already_terminal_on_holon_constructor_call`** —
  `(:wat::holon::Atom "k")` returns AlreadyTerminal.
- **`step_terminal_on_arithmetic_redex`** — sanity that real
  reductions still return StepTerminal, NOT AlreadyTerminal.

Phase 2 (4 walk tests):
- **W1 `walk_w1_chain_to_terminal`** — fully-reducible chain
  `(+ (+ 1 2) 3)`; visit fires per coordinate; final terminal is
  HolonAST::I64(6); accumulator (visit count) ≥ 1.
- **W2 `walk_w2_already_terminal_input`** — already-a-value input
  (`Bind(Atom, Atom)` canonical form); visit fires exactly once
  with step-result = AlreadyTerminal; chain length 0.
- **W3 `walk_w3_skip_short_circuits`** — visitor returns Skip with
  sentinel HolonAST::I64(999); even on a chain that would naturally
  terminate at I64(6), Skip wins.
- **W4 `walk_w4_propagates_eval_step_err`** — `from-watast(quote ...)`
  has no step rule; eval-step! errors; walk propagates as
  `Result::Err`; visitor never sees the error.

---

## What this unblocks

- **Proof 018** — the fuzzy-on-both-stores walker collapses to
  ~10 lines (one visit-fn) around `:wat::eval::walk`. The silent-
  Err-swallow bug becomes structurally impossible because the
  visitor must handle three distinct variants.
- **Lab umbrella 059's L1+L2 cache** — the cache walker uses
  `walk` directly. Skip on cache hit; Continue + record on miss.
  Same shape every chain proof landed on, now substrate-blessed.
- **Future cache-as-coordinate-tree** — every domain that wants
  chain-of-rewrites caching uses `walk`. The trader's enterprise.
  The MTG enterprise. The truth-engine domain. One primitive,
  one visit-fn pattern, one coordinate accounting.

---

## What this arc deliberately did NOT add

- **`max-depth` parameter on `walk`** — Q4 of DESIGN deferred. The
  iterative loop avoids stack growth; pathological infinite chains
  loop forever, but wat-rs's per-call recursion caps inside
  `apply_function` provide a ceiling. Add the parameter when a
  consumer needs deterministic abort.
- **String-rendering helpers for the new variants** — consumers
  format with `:wat::core::show` if they want.
- **A `walk-with-budget` companion** — if step-budget enforcement
  surfaces as a real need, future arc; user-level wat can also
  thread a budget through the accumulator.
- **`presence?` analog of `coincident-explain`** — separate arc
  (per arc 069's deferred list).

---

## The thread

- **Arc 003** — TCO trampoline.
- **Arc 057** — typed HolonAST leaves; `holon_to_watast` /
  `watast_to_holon` round-trip semantics that `try_recognize_holon_value`
  now inverts.
- **Arc 058** — `HashMap<HolonAST, V>` at user level. Cache
  containers.
- **Arc 066** — `eval-ast!` returns wrapped HolonAST per scheme.
  Arc 070's `walk` mirrors the Result-wrap shape.
- **Arc 068** — `:wat::eval-step!`. The single-step primitive.
- **Arc 069** — `coincident-explain`. The diagnostic discipline
  that surfaced this arc's "what does Err mean?" question.
- **Proof 015** — expansion-chain. First reimplementation of the
  walker.
- **Proof 016** — dual-LRU coordinate cache (exact-keyed v4).
  Second reimplementation.
- **Proof 017** — fuzzy-locality cache via `coincident?` (v5).
  Third reimplementation.
- **Proof 018** — fuzzy-on-both-stores. Fourth reimplementation
  silently swallowed `Err`; that bug is what surfaced this arc.
- **2026-04-27 (DESIGN)** — proofs lane drafts arc 070; both
  changes specified.
- **2026-04-27 (Phase 1)** — AlreadyTerminal + classification
  ship (commit 97c827b).
- **2026-04-27 (Phase 2)** — WalkStep + walk + tests + USER-GUIDE
  + INSCRIPTION ship (this commit).
- **Next** — proof 018's walker rewrites around
  `:wat::eval::walk`. Lab umbrella 059 slice 1 starts using it.

PERSEVERARE.
