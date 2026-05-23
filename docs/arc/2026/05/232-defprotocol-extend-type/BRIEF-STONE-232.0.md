# BRIEF — Arc 232 Stone 232.0 — mint `:wat::core::apply` substrate primitive

## What we're doing

Mint `:wat::core::apply` — the universal escape hatch every higher-order Lisp eventually mints. Wat has gone ~3.5 weeks without it because the literal-keyword-dispatch path covered every use case until defprotocol's open polymorphism (arc 232.1+) demanded dynamic-keyword-as-head invocation.

This is Stone 232.0 — the substrate prerequisite that unblocks the rest of arc 232.

## Design substrate (READ FIRST; MANDATORY)

The empirical-disconfirmation finding + the regression-guard probe ARE the design substrate. Read both before drafting any code:

1. **`docs/arc/2026/05/232-defprotocol-extend-type/FINDING-CALL-BY-NAME-GAP.md`** (commit `5c7dddf`) — the substrate gap empirically named; three resolution paths; Option (a) selected
2. **`tests/probe_diagnostic_dynamic_keyword_invocation.rs`** (commit `5c7dddf`) — 3 probes that currently FAIL with `NotCallable { got: "wat::core::keyword" }`; when apply ships these flip FAIL → PASS
3. **`docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md`** (sketches the larger arc 232 plan; relevant for context but THIS stone is just the apply primitive)

The probe IS the working composition pattern. Mirror its shape; don't invent.

## The primitive (Clojure's contract; convergence #16)

```
(:wat::core::apply <head> <a1> <a2> ... <args-vec>) -> :T

  head     : :wat::core::keyword         ;; FQDN of a callable verb/defn
  a1..an   : :T (zero or more leading positional args)
  args-vec : :wat::core::Vector<:T>      ;; LAST arg MUST be a vector; spread as trailing args
  -> :T    : caller annotates (typed-expect pattern; arc 108)
```

Three call shapes, all natural Clojure-idiomatic:

```
;; Pre-built args vector (defprotocol's main case)
(:wat::core::apply :ns::greeting [-> :wat::core::String] ["world"])
;; ≡ (:ns::greeting "world")

;; Mixed: known leading args + tail vector
(:wat::core::apply :ns::add [-> :wat::core::i64] 1 2 [3 4])
;; ≡ (:ns::add 1 2 3 4)

;; Edge: spread everything
(:wat::core::apply :ns::sum [-> :wat::core::i64] [1 2 3 4 5])
;; ≡ (:ns::sum 1 2 3 4 5)
```

**Last positional arg MUST be `:wat::core::Vector<:T>`.** This is the spread constraint. Leading args (a1..an) are passed positionally; vec's elements are appended to form the final argument list passed to `<head>`.

## Implementation surface

### Step 1 — `src/runtime.rs`: `eval_apply` function

Add a new `eval_apply` function modeled on the existing `eval_list` literal-keyword-head dispatch path. Sketched:

```rust
fn eval_apply(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: Span,
) -> Result<Value, RuntimeError> {
    // 1. Arity check: at least 1 arg (the head keyword)
    // 2. Evaluate head; must be Value::wat__core__keyword
    // 3. Evaluate leading args (all but last) as positional
    // 4. Evaluate last arg; must be Value::wat__std__Vec
    //    OR if there is no last arg AND only the head is given, error
    // 5. Spread the vector's elements; concatenate with leading args
    // 6. Look up head's keyword in dispatcher (same lookup paths as
    //    literal-head case: substrate verbs / def-bound values / Symbol-bound fn)
    // 7. Dispatch via same machinery as eval_list literal-head
    // 8. Special-form rejection: if head names :wat::core::defn, :wat::core::let,
    //    :wat::core::if, etc., error with clear diagnostic
    // 9. Wrong-head-type rejection: if head not a keyword, error
}
```

Add the dispatch arm at the top of `eval_list`'s keyword-head match:

```rust
":wat::core::apply" => return eval_apply(args, env, sym, list_span),
```

Place EARLY in the dispatch table (the apply primitive is foundational; matching it before other arms keeps the dispatch predictable).

### Step 2 — `src/check.rs`: register the TypeScheme

Register `:wat::core::apply` in the substrate's TypeScheme table with:

- `type_params: vec![/* generic T */]` — the return type is polymorphic
- `params: vec![keyword_ty()]` — first positional is keyword
- `rest_param_type: Some(/* Vector<T> for the spread tail */)` — variadic with vector spread
- `ret: /* T as polymorphic */`

The exact rest-param mechanics need to mirror existing variadic primitives — check arc 091's `struct->form` or similar for the pattern (look for `rest_param_type: Some(`).

Critical: the rest-param type must encode the "last must be Vector, leading args are T" constraint. If the substrate's TypeScheme can express this directly via rest_param_type, use it. If it requires special-case handling at infer_call time, document the gap and ship the minimal honest type-check (sonnet picks). NO STOP-defer language — ship the cleanest expression of the constraint or surface the gap.

### Step 3 — `tests/probe_diagnostic_dynamic_keyword_invocation.rs`: existing probes flip FAIL → PASS

The 3 existing probes — `probe_1_bound_keyword_invokes_substrate_verb`, `probe_2_runtime_built_keyword_invokes_substrate_verb`, `probe_3_mangled_namespace_invokes_user_defn` — currently FAIL because they use `(verb args)` syntax with a runtime-bound keyword. They need REWRITING to use the new `apply` primitive:

```rust
// Probe 1 BEFORE (FAILS):
(:wat::core::let
  [plus :wat::core::i64::+'2]
  (plus 2 3))

// Probe 1 AFTER (PASSES via apply):
(:wat::core::let
  [plus :wat::core::i64::+'2]
  (:wat::core::apply plus [-> :wat::core::i64] [2 3]))
```

Apply the same rewrite to all 3 probes. The probes become the canonical "apply works for dynamic-keyword invocation" regression guard.

### Step 4 — Add 3 new probes for Clojure-shape coverage

In the same file, add new tests covering Clojure-shape edge cases:

- **`probe_4_apply_with_leading_args_and_tail_vec`** — `(apply :ns::add [-> :i64] 1 2 [3 4])` → 10
- **`probe_5_apply_with_empty_args_vec`** — `(apply :ns::greet [-> :String] [])` → `"hello"` (greet takes no args)
- **`probe_6_apply_rejects_special_form_head`** — `(apply :wat::core::defn ...)` → error with clear diagnostic
- **`probe_7_apply_rejects_non_keyword_head`** — `(apply "not-a-keyword" [...])` → type error or runtime error
- **`probe_8_apply_rejects_non_vector_last_arg`** — `(apply :ns::add 1 2 3)` (no trailing vec) → arity/type error

(Number whatever fits; the brief direction is "Clojure-shape contract is fully bound, not just probed at one case.")

## Verification flow

```
cargo build --release -p wat                          # 0 errors
cargo test --release --lib -p wat --no-fail-fast      # 827 + new lib tests if any, all green
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation -- --nocapture
                                                       # ALL 3 existing probes PASS (was FAIL)
                                                       # 3+ new probes also PASS
cargo clippy --release --lib -p wat -- -D warnings    # 52 warns (baseline match)
git -C /home/watmin/work/holon/holon-rs/ status --short # empty
```

## Out of scope (affirmative scope-bounding)

- **fn-value head** — v2 territory. v1 = keyword-only. When a defservice handler or first-class-fn caller surfaces, v2 adds the fn-value arm in a follow-up stone. Per `feedback_no_known_defect_left_unfixed`: ship the use case, not the speculative surface.
- **defprotocol macro** — that's arc 232.1+. Stone 232.0 mints the substrate primitive ONLY. No defprotocol code in this stone.
- **Reflection layer integration** — `:wat::runtime::lookup-fn` etc. NOT added. apply already handles keyword→callable via dispatcher lookup.
- **holon-rs** — NOT touched.
- **Spread of non-Vector collections** — if last arg is a List or other Seq, spread shape is undefined for v1. Vector-only.
- **Variadic at the head level** — head MUST be a single keyword. No `(apply [k1 k2] ...)` polymorphism.

## STOP triggers (REJECTION criteria — never permission-to-defer)

- **STOP-1:** unexpected compile errors beyond expected substrate edits
- **STOP-2:** any test from baseline (827 passing) goes red post-stone
- **STOP-3:** 120 min elapsed (upper-bound runtime)
- **STOP-4:** holon-rs touched accidentally — REJECTION
- **STOP-5:** clippy `-D warnings` on `src/` adds any NEW warning beyond pre-existing 52
- **STOP-6:** scope creep — fn-value head OR defprotocol macro OR reflection-layer additions
- **STOP-7:** existing 3 probes still FAIL post-stone (the load-bearing flip)
- **STOP-8:** special-form rejection NOT implemented (`(apply :wat::core::defn ...)` would silently dispatch — REJECTION)

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly)
- HARD CUT — no aliases. No `_legacy_name` deprecation shims.
- Per `feedback_inscription_immutable`: do NOT edit past SCORE / FINDING / INSCRIPTION docs; this is forward work in NEW files
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.

## Cross-references

- `FINDING-CALL-BY-NAME-GAP.md` (the empirical evidence that drove this stone)
- `tests/probe_diagnostic_dynamic_keyword_invocation.rs` (the design substrate; 3 probes flip)
- `src/runtime.rs:4015-4050` (literal-keyword head dispatch — mirror this path)
- `src/runtime.rs:4760-7130` (`keyword/from-string` + `keyword::to-string` neighborhood — apply lives nearby)
- `src/check.rs:64-81` (TypeScheme + rest_param_type definition)
- arc 091 slice 8 (runtime quasiquote + struct->form — variadic TypeScheme precedent to mirror)
- arc 108 (typed-expect `-> :T` pattern — apply follows same)
- `feedback_wat_llm_first_design` — LLM-first; Clojure-shape convergence is load-bearing for caller intuition
- `feedback_assertion_demands_evidence` — the probe IS the evidence; the BRIEF asserts only what the probe (post-mint) proves
