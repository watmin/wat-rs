# Arc 068 — Backlog

Pre-implementation work plan + decision register. DESIGN.md is the
reasoning artifact; this file is the build sheet.

---

## Decisions taken (DESIGN.md Q1-Q9)

- **Q1 — input/output AST type.** WatAST in; WatAST out for
  `StepNext`; HolonAST out for `StepTerminal`. Matches arc 066's
  shape on the terminal side.
- **Q2 — effectful ops.** Reject with
  `EvalError::EffectfulInStep { op: String }`. Consumer falls back
  to `eval-ast!` for effectful sub-forms.
- **Q3 — forms with no step rule.** Reject with
  `EvalError::NoStepRule { op: String }`. Consumer chooses fallback.
- **Q4 — substitution.** Textual; the type checker guarantees no
  capture. No α-renaming.
- **Q5 — match arm selection.** One step does arm selection +
  pattern-binding substitution. Reuse `eval_match`'s logic.
- **Q6 — lambda values.** `StepTerminal` carries
  `HolonAST::Atom(<canonical-form>)` — opaque-identity wrap per
  Chapter 54 / Story 1.
- **Q7 — span preservation.** Yes; preserve outer-form span through
  rewrites. Errors carry the failing form's span.
- **Q8 — multi-binding `let*`.** Peel one binding per step.
- **Q9 — capacity-mode interaction.** Capacity errors propagate as
  `Err(EvalError::CapacityExceeded { ... })`. Same as `eval-ast!`.

## Decisions deferred (DESIGN.md "Open questions")

- Macroexpand-stepping → future arc.
- Effectful stepping (StepEffect variant) → future arc when a
  consumer surfaces a need.
- Step-budget primitive → lab-userland helper, not substrate.
- Dual-LRU cache library (`wat/std/eval/cache.wat`) → ships after
  this arc lands and proof 016 v4 confirms the surface.
- Multi-walker cooperation patterns → consumer-side, not substrate.
- Reckoner integration → downstream lab work (BOOK Chapter 55).

---

## Slice plan

### Slice 1 — eval-step! (THIS ARC)

The single slice. If the impl surfaces a natural split, fork into
sub-slices documented here.

**Files touched:**

- `src/runtime.rs`
  - Add `Value::eval__StepResult(Arc<StepResult>)` variant.
  - Add `StepResult` enum (Rust-side):
    `enum StepResult { StepNext(Arc<WatAST>), StepTerminal(Arc<HolonAST>) }`.
  - Add `EvalError::EffectfulInStep { op: String }` and
    `EvalError::NoStepRule { op: String }` variants.
  - Add `eval_form_step(args, env, sym) -> Result<Value, RuntimeError>`
    that mirrors `eval_form_ast`'s wrap-as-Result shape; calls into
    new `step` function.
  - Add `step(form: &WatAST, env: &Environment, sym: &SymbolTable)
    -> Result<StepResult, RuntimeError>` — the core stepper.
  - Add per-form-shape step rules (see DESIGN's table).
  - Wire `:wat::eval-step!` keyword in the form dispatcher (next to
    `:wat::eval-ast!`).
  - Tests in `mod tests` per DESIGN Q10.
- `src/freeze.rs`
  - Register `:wat::eval::StepResult` enum at frozen-world setup
    (mirror Option/Result registration).
- `docs/USER-GUIDE.md`
  - New row in the eval-family section for `:wat::eval-step!`.
  - New row for `:wat::eval::StepResult`.
  - Document the EvalError variants.
  - Add a worked example (~30 lines) showing the dual-LRU cache
    loop. Cite BOOK Chapter 59.

**Tests (inline `mod tests`):**

1. `step_lit_i64` — `5` → `StepTerminal HolonAST::I64(5)`.
2. `step_lit_f64`, `step_lit_bool`, `step_lit_string` — same shape.
3. `step_arith_single_redex` — `(+ 2 2)` →
   `StepTerminal HolonAST::I64(4)`.
4. `step_arith_left_descent` — `(+ (+ 1 2) 3)` →
   `StepNext (+ 3 3)`; one more step → `StepTerminal HolonAST::I64(6)`.
5. `step_arith_right_descent` — `(+ 5 (+ 1 2))` →
   `StepNext (+ 5 3)`; one more → `StepTerminal HolonAST::I64(8)`.
6. `step_let_star_substitute` — `(let* ((x :i64 5)) (* x x))` →
   `StepNext (* 5 5)`; one more → `StepTerminal HolonAST::I64(25)`.
7. `step_let_star_peel_first` — `(let* ((a :i64 (+ 1 1))
   (b :i64 a)) b)` peels `(+ 1 1)` first; verify each step.
8. `step_if_branch_true` — `(if true 1 0)` → `StepNext 1` →
   `StepTerminal HolonAST::I64(1)`.
9. `step_if_branch_false` — `(if false 1 0)` → `StepNext 0` →
   `StepTerminal HolonAST::I64(0)`.
10. `step_if_cond_reduces` — `(if (= 1 1) 1 0)` → `StepNext (if true
    1 0)` → `StepNext 1` → `StepTerminal`.
11. `step_match_canonical` — `(match (Some 5) ((Some n) n) (:None 0))`
    → `StepNext 5` → `StepTerminal HolonAST::I64(5)`.
12. `step_match_scrutinee_reduces` — `(match (+ 1 1) (n n))` →
    `StepNext (match 2 (n n))` → `StepNext 2` → `StepTerminal`.
13. `step_user_function_call` — define `(square (n :i64) -> :i64
    (* n n))`; `(square 3)` → `StepNext (* 3 3)` → `StepTerminal
    HolonAST::I64(9)`.
14. `step_tail_recursion` — define `(sum-to (n :i64) (acc :i64) ->
    :i64) (if (= n 0) acc (sum-to (- n 1) (+ acc n)))`; step
    `(sum-to 3 0)` until `StepTerminal HolonAST::I64(6)`. Assert
    step count ≤ 30 (TCO check).
15. `step_holon_constructor_atom` — `(:wat::holon::Atom "k")` →
    `StepTerminal HolonAST::Atom(HolonAST::String("k"))`.
16. `step_holon_constructor_bind` — `(:wat::holon::Bind (Atom "k")
    (Atom "v"))` reduces args first, fires Bind on the third step.
17. `step_holon_thermometer` — `(:wat::holon::Thermometer 0.5 0.0
    1.0)` → `StepTerminal HolonAST::Thermometer { value: 0.5, ... }`.
18. `step_effectful_send_rejected` — set up channel; step
    `(:wat::kernel::send chan 1)` →
    `Err(EvalError::EffectfulInStep { op: ":wat::kernel::send" })`.
19. `step_round_trip_agrees_with_eval_ast` — pick five forms; step
    each to terminal; compare with `eval-ast!` result; assert
    coincidence.
20. `step_span_preserved` — step `(+ (+ 1 2) 3)`; assert the outer
    `+` form's span survives the inner reduction.

**Acceptance criteria:**

- All 20 tests pass.
- `cargo clippy` clean.
- USER-GUIDE.md compiles (no broken cross-references).
- One commit; arc 068 marker in commit message.
- INSCRIPTION.md committed in the same arc (the post-ship record;
  see arc 066's INSCRIPTION.md for the template).

**Estimated effort:** 6-10h focused. Most of the cost is the
per-form step rules (about 12 of them); each is small but each
needs its own test case.

---

## Risks / unknowns

- **Step rule coverage gaps.** The DESIGN's table lists ~12 form
  shapes with rules. Wat has more shapes than that. v1 ships the
  core; ops without rules return `NoStepRule`. **Mitigation:** the
  consumer (proof 016 v4) drives. If a needed form isn't covered,
  this arc extends; otherwise, future arcs.
- **Substitution interaction with shadowing in `let*` and `match`.**
  Q4 settled on textual substitution. The type checker enforces
  unique resolution per binding; textual substitution should be
  safe. **Mitigation:** test 7 (`step_let_star_peel_first`) and a
  match test with shadowing fields covers this.
- **Lambda Q6 implementation.** Wrapping a lambda as
  `HolonAST::Atom(<canonical-form>)` requires lowering the
  lambda's WatAST to canonical bytes. The lowering exists (per
  arc 057's `watast_to_holon`); using it for lambdas may surface
  edge cases (closures over outer bindings, etc.). **Mitigation:**
  start with bare lambdas (no closure captures); higher-order
  programs that need real closures get a future arc. Test 13 covers
  the bare case.
- **Span propagation through substitution.** When a value substitutes
  for a variable, whose span does the result carry? The substituted
  value's span (because it's the value that ended up there) or the
  variable's span (because that's where the rewrite happened)?
  **Recommendation:** the substituted value's span. The variable
  is gone after substitution; the value's origin is preserved.
  Test 20 verifies the outer form's span.

---

## Definition of done

- DESIGN.md frozen (no further edits except the Status header
  flip from "PROPOSED" to "shipped 2026-MM-DD").
- INSCRIPTION.md written (post-ship record; see arc 066 template).
- All tests in slice 1 pass.
- `cargo test --workspace` clean.
- USER-GUIDE.md surface row added.
- Commit pushed.
- Linked from BOOK Chapter 59 (or follow-on chapter naming the
  shipping).

After done: ping consumer (proof 016 v4) to begin its rewrite.
