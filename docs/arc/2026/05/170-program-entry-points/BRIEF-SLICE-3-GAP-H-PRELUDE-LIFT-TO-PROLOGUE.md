# Arc 170 slice 3 Gap H BRIEF — closure-extraction lifts fn-body prelude forms into prologue

**Sonnet.** Substrate fix that unblocks Path E macro shape (Gap G's blocked shape). Discovered by Gap G's attempted rewrite — `(:wat::core::define ...)` at expression position inside a fn body's `do` triggers `RuntimeError::DefineInExpressionPosition`. This Gap extends `extract_closure` to LIFT prelude forms (defines, structs, enums) from the fn body's leading do-prefix INTO the closure's `prologue`.

User direction 2026-05-13: A-wide selected. Quote: *"closure-extraction lifts prelude `define`s into prologue — preserves the single mental model 'define = top-level registration'... reuses startup_from_forms."*

## Backstory in one sentence

After Gap H ships, the next deftest-hermetic Path E retry uses identical shape as Gap G's blocked attempt; substrate now lifts the defines to where they're processed correctly.

## Goal — extract_closure lifts top-of-body forms to prologue

Today: `extract_closure` builds a `ClosurePackage { prologue, fn_body }` where the body is the fn's `body` AST verbatim. If the body is a `do` whose children start with `define`/`struct`/`enum` forms (the "prelude" pattern), those forms reach the child's `eval()` at expression position and get rejected.

Target: extract_closure detects the leading prelude run inside the fn body's `do` (consecutive forms whose head keyword is `:wat::core::define` / `:wat::core::struct` / `:wat::core::enum`); appends those forms to `prologue` (after F-3's type registry inheritance); strips them from the body's `do`. The remaining `do` is pure expressions; child's `eval_do_tail` accepts it. Child's `startup_from_forms(prologue)` processes the lifted forms at step 6 just like outer top-level — registering them in the child's `SymbolTable` + `TypeEnv` before the body runs.

## Why A-wide (vs A-narrow)

Per sonnet's recommendation + user confirmation:

- **A-narrow** (runtime local-env-frame registration): `define` means different things in different positions. Bigger conceptual addition.
- **A-wide** (closure-extraction lift): `define = top-level registration` stays the single mental model; the lift moves the define to where top-level processing happens. Reuses `startup_from_forms`.

A-wide is the cleaner doctrine.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-G-PATH-E-ISOLATION.md`** (commit `021884a`) — the blockage analysis + sonnet's A-narrow/A-wide refinement
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`** — V4 root cause + failure patterns
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-F-3-CLOSURE-TYPE-REGISTRY.md`** — Gap F-3 extended extract_closure for type registry; this Gap extends it further for prelude lift
4. **`src/closure_extract.rs`** — current `extract_closure`; F-3's type-registry sweep at the bottom (this Gap adds the prelude-lift sweep alongside)
5. **`src/runtime.rs` `eval()` / `eval_do_tail`** — locate `DefineInExpressionPosition` rejection
6. **`src/freeze.rs` `startup_from_forms`** — the prologue-processing pipeline (step 6 registers defines)

## Implementation path

### Phase 1 — Locate + understand `extract_closure`'s body-walking (5-10 min)

Read `extract_closure` end-to-end. Identify where the fn body is captured. Locate F-3's whole-registry type-registry sweep — Gap H adds a parallel sweep.

### Phase 2 — Probes (failing baseline; 15-20 min)

Create `tests/probe_closure_body_prelude_lift.rs` with probes that demonstrate the failure today + the fix:

```rust
#[test]
fn probe_define_in_fn_body_do_prefix_lifts_to_prologue() {
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [proc
              (:wat::kernel::spawn-process
                (:wat::core::fn [_rx _tx] -> :wat::core::nil
                  (:wat::core::do
                    (:wat::core::define (:body::helper -> :wat::core::i64) 42)
                    (:wat::core::let [v (:body::helper)] ()))))]
            ()))
    "#;
    // Today: child fails with DefineInExpressionPosition
    // After fix: child succeeds; :body::helper registers in child's SymbolTable
}

#[test]
fn probe_struct_in_fn_body_do_prefix_lifts_to_prologue() {
    // Struct inside fn body do — same lift; child registers at its top-level
}

#[test]
fn probe_enum_in_fn_body_do_prefix_lifts_to_prologue() {
    // Enum lift mirror
}

#[test]
fn probe_mixed_prelude_lift() {
    // struct + enum + define all at do's prefix; all lift in order
}

#[test]
fn probe_prelude_prefix_terminates_at_first_expression() {
    // Defines BEFORE first expression lift; defines AFTER first expression don't
    // (they remain in body and would still fail eval; consistent with "do-prefix")
    // Verify the prefix-terminating semantic
}
```

Probes confirm failure baseline; after fix all pass.

### Phase 3 — Extend `extract_closure` (15-30 min)

Add a pre-body sweep:

```rust
// After F-3's type-registry sweep:
if let Some(do_children) = is_fn_body_do(&fn_body) {
    let (prelude_forms, residual_body) = split_prelude_prefix(do_children);
    for prelude_form in prelude_forms {
        prologue.push(prelude_form);  // appended after type-registry sweep's entries
    }
    fn_body = reconstruct_fn_body(residual_body);
}
```

`split_prelude_prefix`: walks children left-to-right; collects forms whose head is `:wat::core::define` / `:wat::core::struct` / `:wat::core::enum`; stops at first non-prelude form; returns `(prelude, rest)`.

`is_fn_body_do`: returns `Some(children)` if the fn body IS a do form; otherwise None (don't lift if body isn't a do).

### Phase 4 — Order in prologue

Prologue order matters at child startup. F-3's type-registry sweep adds inherited types FIRST (so child sees parent's types). Gap H's lifted prelude appends AFTER (so child sees: parent's types → test's prelude types/helpers → body).

Alternative: order doesn't matter because child's `startup_from_forms` re-topologizes. Verify; surface in SCORE.

### Phase 5 — Verify

```bash
# Gap H probes
cargo test --release --test probe_closure_body_prelude_lift 2>&1 | tail -5

# All existing substrate probes still pass
cargo test --release --test probe_do_splice_def probe_let_splice_def probe_do_splice_define probe_let_splice_define probe_do_splice_struct probe_do_splice_enum probe_let_splice_struct probe_let_splice_enum probe_spawn_process_parent_type probe_resolver_quote_awareness probe_deftest_hermetic_isolation 2>&1 | tail -5

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2227 + N / 0 failed (N = number of new probes; expect 5)
```

## Scope (what's IN)

- `extract_closure` extended with prelude-lift sweep
- `is_fn_body_do` + `split_prelude_prefix` helper fns
- 5+ probes proving prelude forms lift correctly
- Workspace stays at 0 failed

## Scope (what's OUT)

- deftest-hermetic Path E macro rewrite — separate slice (Gap G round 2, or fold into Phase E V5)
- Phase E V5 deftest Path A rewrite — separate slice
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system
- Changes to `eval()` / `eval_do_tail` — substrate KEEPS rejecting `define` at expression position; this slice ensures define forms never reach that position via the lift

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `extract_closure` extended with prelude-lift sweep | grep + read |
| B | `is_fn_body_do` + `split_prelude_prefix` helpers exist | grep + read |
| C | 5+ probes pass: define / struct / enum / mixed / prefix-terminating | cargo test |
| D | All 25 prior substrate probes still pass | cargo test |
| E | `cargo check --release` green; workspace at 2227 + N / 0 failed | full test |
| F | F-3's type-registry sweep still functions correctly (no regression) | F-3 probes pass |

**6 rows.** All must PASS.

## Predicted runtime

**45-90 min sonnet.** Pattern-mirrors Gap F-3's extract_closure extension; adds a parallel sweep + helper functions. Probe writing is the load-bearing time.

**Hard cap:** 180 min (2×).

## Constraints (hard)

- DO NOT modify `eval()` / `eval_do_tail` — substrate keeps rejecting `define` at expression position. The fix is upstream (lift before eval ever sees it).
- DO NOT add new substrate features beyond the helper fns + the lift in extract_closure
- DO NOT modify the deftest-hermetic macro body (separate slice)
- DO NOT modify any test call site outside the new probe file
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- If existing deftest-hermetic users (ambient-stdio.wat etc.) break under the lift, STOP and report — the lift should be transparent for current callers (their preludes go through forms-quoted AST which already reaches child's top-level via run-sandboxed-hermetic-ast)

## Honest delta categories (anticipated)

1. **Prelude-prefix-termination semantics** — at first non-prelude form OR scan whole body? Sonnet picks; surface rationale (probably first non-prelude per arc 168 multi-form body convention).
2. **Prologue ordering** — types-first (F-3) then lifted-defines, OR interleave? Verify topological needs.
3. **Interaction with F-3's type-registry sweep** — both extend extract_closure; should they share a helper or stay distinct?
4. **Body-shape edge cases** — fn body that ISN'T a do (single expression); fn body that's a let containing defines; surface what cases are in/out of scope.
5. **Anything unexpected** — particularly around closure-capture interaction (lifted defines mustn't reference body-local let bindings; if they do, that's the user's bug, but the diagnostic should be helpful)

## Cross-references

- `021884a` Gap G (the blockage that revealed this gap; probes proving isolation contract are already in tests/probe_deftest_hermetic_isolation.rs)
- `fe06bb1` Gap F-3 (extract_closure extension precedent for type-registry inheritance)
- `f9c8aef` Gap F-1 (struct/enum pre-registration in top-level do/let — parallel concern at parent scope)
- `662f5bc` Gap F-2 (resolver quote-awareness — composes with this fix's prelude-lift)
- After Gap H ships: deftest-hermetic Path E macro shape rewrite becomes a small wat/test.wat edit (separate slice or folded into V5)
