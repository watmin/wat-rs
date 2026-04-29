# Arc 102 — `:wat::eval-ast!` polymorphic return — INSCRIPTION

**Status:** shipped 2026-04-29.

`:wat::eval-ast!`'s scheme changed from
`Result<:wat::holon::HolonAST, :wat::core::EvalError>` to
`Result<:T, :wat::core::EvalError>` polymorphic. The runtime
now returns the bare inner Value (reverts arc 066's
`value_to_holon` wrap). Caller annotates `T` with the type they
expect — same trust-the-caller discipline as `:wat::edn::read`
/ `:wat::eval-edn!`.

**Predecessor:** [arc 066](../066-eval-ast-wraps-holon/INSCRIPTION.md).
Arc 066 diagnosed the scheme-vs-runtime lie correctly and fixed
it by deforming the runtime to match the scheme. Arc 102 picks
the opposite fix — deforms the scheme to match the runtime.

**Surfaced by:** arc 093 slice 3 design conversation 2026-04-29.
The telemetry-interrogation flow needed to lift a `data` column
to `Value::Struct` for the Clara matcher (arc 098) to consume.
Arc 066's wrap stood between row bytes and the typed Value with
no clean extraction step — and the user spotted the half-built
shape:

> "this appears like we half built a thing and are just realizing
> we need to fully build it?... the questions - what is simple?
> what is honest? what is a good ux?"

> "option 1 is the only option i see"

The arc closed in two slices same day.

---

## What shipped

### Slice 1 — code reversal

`src/runtime.rs::eval_form_ast` drops the `value_to_holon` wrap.
`run_constrained`'s inner Value flows straight into
`wrap_as_eval_result`'s `Result::Ok` slot.

`src/check.rs::eval-ast!` scheme: `type_params: vec!["T".into()]`,
return `Result<:T, :EvalError>`. The `T` unifies with whatever
the caller binds the result to.

Five substrate unit tests migrated:

- `eval_ast_wraps_i64_result_as_holon_leaf` →
  `eval_ast_returns_bare_i64_result`
- `eval_ast_wraps_bool_result_as_holon_leaf` →
  `eval_ast_returns_bare_bool_result`
- `eval_ast_wraps_string_result_as_holon_leaf` →
  `eval_ast_returns_bare_string_result`
- `eval_ast_rejects_non_holon_expressible_result` →
  `eval_ast_passes_through_vec_result` (Vec results no longer
  error — the wrap that couldn't handle them is gone)
- `eval_ast_passes_through_holon_result` — unchanged (still
  passes; T = HolonAST and the runtime IS a HolonAST when the
  inner form produced one)

Plus three call-site annotation updates: `(Ok h) atom-value h`
→ `(Ok n) n` for callers that want the bare value directly.

`cargo test --workspace` green; 737 lib tests pass.

### Slice 2 — docs

This INSCRIPTION + the DESIGN that recorded the reversal +
USER-GUIDE updates (forms-table entry rewritten; §6 inline
example annotates `T`) + 058 FOUNDATION-CHANGELOG row in the
lab repo.

---

## Tests

`cargo test --workspace`: 737 substrate lib tests + every
integration test still green. Migration touched ~70 lines of
runtime.rs + check.rs + the unit-test suite; no external
consumer breakage (the only callers are wat-tests + wat-cli
tests; both live inside the repo).

---

## What's NOT in this arc

- **Removing `value_to_holon`.** The Rust helper stays public
  for callers that explicitly want to lift a `Value` to a
  HolonAST. They invoke it themselves now instead of the
  runtime doing it on their behalf for every `eval-ast!`.
- **Touching `eval-step!` / `eval-edn!` / etc.** `eval-edn!`
  was already polymorphic (arc 086). `eval-step!` returns a
  custom `StepResult` enum that doesn't have the same scheme-
  vs-runtime tension. No follow-on arcs needed on this front.
- **`atom-value` changes.** Still the unwrap for HolonAST
  primitive leaves. Callers binding `T = HolonAST` to extract
  a primitive use this exactly as before.

---

## Lessons

1. **"Fix the lie" has a direction.** When a static scheme
   disagrees with runtime, you can fix either side. Arc 066
   picked "fix runtime to match scheme" (wrap every result).
   Arc 102 picks "fix scheme to match runtime" (polymorphic
   return). Both fixes are legitimate; the right choice depends
   on what's natural at the runtime boundary. *Eval naturally
   produces a value; making the scheme honest about that beats
   making every result conform to a universal-carrier shape
   the caller didn't ask for.*

2. **Half-built primitives surface late.** Arc 066 shipped
   without a real consumer who needed Value-as-Struct out of
   eval-ast!. Arc 093's interrogation flow was that consumer,
   four months later. The wrap that "fixed" arc 028's lie made
   the new use case impossible. *When fixing a scheme-vs-
   runtime lie, the choice of direction has to consider the
   shape of future callers, not just the current ones.*

3. **`Result<:T, :EvalError>` is the wat substrate's settled
   pattern.** `:wat::edn::read` (arc 086), `:wat::eval-edn!`
   (arc 028 carried forward), and now `:wat::eval-ast!` all
   share it. Trust-the-caller polymorphism: caller annotates
   the type they expect, runtime returns whatever, downstream
   ops fail at runtime if the expectation was wrong. Same
   shape Clojure / Lisp / dynamically-typed-with-static-hints
   languages have used for decades. *When a substrate primitive's
   output shape depends on its input value (not just input type),
   polymorphic `:T` is the right default.*

4. **Three-question discipline catches half-built shapes.**
   *"What is simple? What is honest? What is a good UX?"* —
   applied to arc 066's wrap, all three questions point the
   other way. The discipline doesn't tell you the answer; it
   surfaces when you're avoiding one. Arc 102 is the answer
   that all three questions agreed on.

5. **Reversal arcs are honest, not embarrassing.** Arc 066 was
   ~2 weeks before arc 102; arc 066's INSCRIPTION says *"`Pre-arc-066
   the result was the bare Value`"* — preserving the original
   shape in the substrate's own history. Arc 102 doesn't
   "undo" arc 066 — it builds on arc 066's diagnosis with a
   different fix. The substrate is honest about its own design
   history; arcs that reverse arcs are part of that.

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "this appears like we half built a thing and are just realizing
> we need to fully build it?..."

> "the questions - what is simple? what is honest? what is a
> good ux?"

> "option 1 is the only option i see"

The arc closed when slice 1's `cargo test --workspace` came
back green and slice 2's docs landed minutes later. The
substrate is what the user said it should be when he named it.

**PERSEVERARE.**
