# Arc 102 — `:wat::eval-ast!` polymorphic return — DESIGN

**Status:** SETTLED — opened and closed in the same conversation
turn 2026-04-29 mid-arc-093 slice 3 design. The decision was a
small substrate reversal with no design questions; this DESIGN
exists to record what arc 066 got wrong and how arc 102 corrects
the shape.

**Predecessor:** [arc 066](../066-eval-ast-wraps-holon/INSCRIPTION.md).

---

## What arc 066 fixed and what it broke

Pre-arc-066 (arc 028's surface):

- Scheme: `:wat::eval-ast! :wat::WatAST -> :Result<HolonAST, EvalError>`.
- Runtime: returned the bare Value (e.g. `Value::i64(4)` for `(+ 2 2)`).
- Mismatch: caller pattern-matching `(Ok h)` got `h` typed-as-HolonAST
  per the scheme but actually a `Value::i64` at runtime. Calling
  `(:wat::core::atom-value h)` on this raised `TypeMismatch`.

Arc 066's diagnosis:

> "eval-ast!'s scheme lied about its runtime behavior."

Arc 066's fix: deform the **runtime** to match the scheme. Wrap
every inner Value as `Value::holon__HolonAST` (via
`value_to_holon`) so the static `Result<HolonAST, EvalError>`
type was honest at the runtime boundary. Forms whose terminal
value couldn't be wrapped (Vec, Tuple, channels) returned `Err`.

That fixed the immediate mismatch. But the wrap was forced —
"eval an AST" naturally produces a value, whatever its shape;
universal-HolonAST-wrap is a layer the caller didn't ask for.
Arc 093 surfaced this when the telemetry-interrogation flow
needed to lift a `data` column to a `Value::Struct` for the
Clara matcher (arc 098) to consume — there's no way through
arc 066's wrap without a second extraction step that doesn't
exist for struct-shape values.

---

## What arc 102 does instead

Same diagnosis, opposite fix: deform the **scheme** to match
the runtime.

```text
Pre-arc-102 (arc 066 status quo):
  :wat::eval-ast! :wat::WatAST -> :Result<:wat::holon::HolonAST, :wat::core::EvalError>
  Runtime: wraps inner Value via value_to_holon

Post-arc-102:
  :wat::eval-ast! :wat::WatAST -> :Result<:T, :wat::core::EvalError>
  Runtime: returns inner Value bare
```

The polymorphic `:T` follows the trust-the-caller discipline that
`:wat::edn::read` / `:wat::eval-edn!` already use. Caller binds
the result with the type they expect; the type checker unifies
`T` with that type; runtime returns whatever it produces;
type-mismatched downstream ops fail at runtime in the same way
they do for any other polymorphic-return primitive.

---

## Three-question discipline

Applied to the wrap decision:

**Simple?** One primitive returning the bare value beats arc
066's wrap layer plus the second-extraction primitive arc 093
would have needed. Polymorphic `:T` keeps the surface flat; no
sibling `eval-ast-value!` to learn alongside `eval-ast!`.

**Honest?** "Evaluate an AST" produces a value, not a HolonAST-
wrapped value. The wrap was a workaround for a typing-vs-runtime
mismatch; the natural answer was to fix the typing, not deform
the runtime to match the broken typing. Arc 066 picked the
workaround; arc 102 picks the fix.

**Good UX?** Polymorphic `Result<:T, :EvalError>` matches every
consumer's actual usage: bind with the type you expect, pattern-
match Ok, get the value. The HolonAST-wrap path arc 066 forced
on every caller was extra work for everyone except those who
explicitly wanted a HolonAST.

---

## What changes

### Substrate — `src/runtime.rs::eval_form_ast`

Drop the `value_to_holon` wrap. Return `run_constrained`'s
inner Value directly (wrapped in `Result::Ok` by
`wrap_as_eval_result` as before).

### Substrate — `src/check.rs`

Change `eval-ast!`'s scheme: `type_params: vec!["T".into()]`,
return `Result<:T, :EvalError>`.

### Test migration

Five substrate unit tests verified arc 066's specific wrap
behavior. Update them to verify arc 102's bare-Value:

- `eval_ast_wraps_i64_result_as_holon_leaf` →
  `eval_ast_returns_bare_i64_result`. Caller binds T = i64;
  `(Ok n) n` returns the i64 directly (no atom-value extraction).
- `eval_ast_wraps_bool_result_as_holon_leaf` →
  `eval_ast_returns_bare_bool_result`.
- `eval_ast_wraps_string_result_as_holon_leaf` →
  `eval_ast_returns_bare_string_result`.
- `eval_ast_rejects_non_holon_expressible_result` →
  `eval_ast_passes_through_vec_result`. Vec results no longer
  error (the wrap that couldn't handle them is gone); they
  flow through cleanly when the caller binds T = `:Vec<i64>`.
- `eval_ast_passes_through_holon_result` — unchanged (still
  works; T = HolonAST and the runtime IS a HolonAST when the
  inner form produced one).

Plus three call-site updates that pattern-matched `(Ok h)` then
called `(:wat::core::atom-value h)`: change to `(Ok n) n` since
the bare value comes out directly.

### USER-GUIDE

Forms-table entry for `:wat::eval-ast!` reflects the new
polymorphic return + the trust-the-caller discipline. The
inline §6 "Story 2 — value" example annotates the expected `T`
at the binding.

---

## What does NOT change

- **Arc 066's diagnosis.** *"The scheme lied about runtime
  behavior"* is still right — arc 102 just picks the other
  fix. The scheme that lied was `Result<HolonAST, EvalError>`;
  the new scheme `Result<:T, :EvalError>` is honest because
  `T` IS what the runtime returns.
- **`value_to_holon`** as a Rust helper. Arc 066's wrap fn
  stays public for callers that explicitly want the HolonAST
  shape. They invoke it themselves now instead of the runtime
  doing it on their behalf.
- **`atom-value`** primitive. Still the unwrap for HolonAST
  primitive leaves. Callers binding `T = HolonAST` to extract
  a primitive use this exactly as before.
- **`:wat::eval-edn!` / `:wat::edn::read`.** Already polymorphic
  per arc 086. Arc 102 brings `eval-ast!` into the same shape;
  no change to the EDN-side primitives.
- **Lab repo.** No `holon-lab-trading` source code calls
  `eval-ast!` directly (only BOOK references). No migration
  needed.

---

## Tradeoffs

### Soundness (or its absence)

Polymorphic `Result<:T, :EvalError>` is unsound in the strict
type-system sense — the compiler can't enforce that the inner
eval's output matches the caller's annotated `T`. If a user
binds `T = :i64` but the AST evaluates to a String, the
pattern-match `(Ok s)` succeeds and `s` is the String, but the
binding's static type is i64; downstream operations on `s`
fail at runtime with a `TypeMismatch`.

This is the same shape `:wat::edn::read` (parse + bridge to
runtime Value via the type registry) already accepts. The wat
substrate has settled on this trust-the-caller pattern for
operations whose result type can't be statically inferred from
the operation's input. Arc 102 brings `eval-ast!` into the
same family.

### Migration cost

Five substrate unit tests, three call-site annotations
(`(Ok h) atom-value h` → `(Ok n) n`). Total ~30 lines touched.
No external-consumer breakage — wat-tests + wat-cli/tests are
the only callers, and they all live inside this repo.

If a downstream consumer somewhere had `(Ok h)` pattern-matches
relying on `h` being a HolonAST, they'd see runtime
TypeMismatch on subsequent ops. Migration: change the binding's
annotated `T` to the actual eval result type, or wrap with
`(value-as-holon-ast result)` if they specifically want the
HolonAST shape.

### Why not also fix `eval-edn!` to take a type-witness?

`eval-edn!` was already polymorphic per arc 086. The shape was
arc 102's eventual destination — arc 102 just brings
`eval-ast!` into line. No further change needed for `eval-edn!`;
no future arc on this front anticipated.

---

## Slice plan

**Slice 1** — code reversal — *shipped 2026-04-29*.

- `src/runtime.rs::eval_form_ast`: drop the wrap.
- `src/check.rs`: change scheme to polymorphic.
- Migrate 5 substrate unit tests + 3 call-site annotations.
- `cargo test --workspace` green.

**Slice 2** — docs — *shipped 2026-04-29*.

- This DESIGN.
- INSCRIPTION sealing the arc.
- USER-GUIDE forms-table entry update + §6 inline example.
- 058 FOUNDATION-CHANGELOG row in the lab repo.

---

## Predecessors / dependencies

**Reverses (in part):**
- Arc 066 — `eval-ast! returns wrapped HolonAST`. Arc 102
  reverses the wrap; the diagnosis arc 066 made remains
  correct.

**Sibling pattern:**
- Arc 086 — `:wat::edn::read` polymorphic `Result<:T, :EvalError>`
  return. Arc 102 brings `eval-ast!` into the same family.

**Surfaced by:**
- Arc 093 slice 3 design — telemetry interrogation needed to
  lift a `data` column to `Value::Struct` for the Clara matcher
  (arc 098) to consume. Arc 066's wrap stood between row bytes
  and the typed Value; arc 102 removes that obstacle.

## What this enables

- **Arc 093 slice 3 ships as a thin wat-side define** —
  `Event::Log/data-value` does pattern-match → newtype unwrap
  → `eval-ast!` → bare `Value::Struct`, ready for `matches?`.
- **Future evaluators** that produce arbitrary runtime shapes
  (e.g., a sandboxed eval primitive) inherit the same surface
  — polymorphic `Result<:T, :EvalError>`, T set by the caller's
  binding annotation.

**PERSEVERARE.**
