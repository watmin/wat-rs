# Arc 170 slice 3 Gap H SCORE — closure-extraction lifts fn-body prelude forms into prologue

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2232 passed / 0 failed

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `extract_closure` extended with prelude-lift sweep (in `src/closure_extract.rs`) | grep + read | PASS — `split_body_prelude(rewritten_body)` call at line 309; `body_prelude_forms` appended to prologue at step 4 (line 350) |
| B | `is_prelude_form` + `split_body_prelude` helper fns exist; `walk_struct_form` + `walk_enum_form` added as required companions | grep + read | PASS — all four helpers in `src/closure_extract.rs` (lines 1762, 1802, 799, 833) |
| C | 5+ probes pass: define / struct / enum / mixed / prefix-terminating semantics | cargo test | PASS — `5 passed; 0 failed` (`tests/probe_closure_body_prelude_lift.rs`) |
| D | All 25 prior substrate probes still pass | cargo test | PASS — all 11 prior probe test files pass; no regressions |
| E | `cargo check --release` green; workspace at 2227 + 5 / 0 failed | full test run | PASS — `2232 passed; 0 failed` |
| F | F-3's type-registry sweep still functions correctly; F-3 probes pass | F-3 probe re-run | PASS — `probe_spawn_process_parent_type`: 3/3 pass |

**All 6 rows PASS.**

---

## Files changed

| File | Change |
|------|--------|
| `src/closure_extract.rs` | Four additions: `is_prelude_form` + `split_body_prelude` helpers; `walk_struct_form` + `walk_enum_form` walkers; `split_body_prelude(rewritten_body)` call in `extract_closure`; prologue step 4 loop emitting `body_prelude_forms`. `walk_free_symbols` List arm extended with `:wat::core::struct` and `:wat::core::enum` dispatch cases. |
| `tests/probe_closure_body_prelude_lift.rs` | CREATED — 5 Gap H probes (define / struct / enum / mixed / prefix-terminating). |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-H-PRELUDE-LIFT-TO-PROLOGUE.md` | CREATED — this file. |

---

## Workspace delta

- Baseline (post-Gap-G): 2227 passed / 0 failed
- Post-Gap-H: 2232 passed / 0 failed (+5 probes)

---

## Prelude-prefix-termination rationale

`split_body_prelude` uses `take_while(|child| is_prelude_form(child)).count()` — a strict prefix scan that stops at the FIRST non-prelude child. This is the correct semantic for three reasons:

1. **Mental model consistency**: "define = top-level registration" (A-wide doctrine). A define that appears after an expression is presumably meant as dead code or the user's mistake; the substrate already rejects it with `DefineInExpressionPosition`. The lift does not attempt to rescue all defines throughout the body — only the PREFIX run before any expression.

2. **Determinism**: the prefix-length is defined by the first non-prelude child, not by scanning the whole body. No ambiguity when a define appears mid-body after an expression.

3. **Gap G precedent**: the existing `preregister_fn_defs_in_do` (Gap F-1) similarly scans a do-body's prefix for struct/enum before registering them. The prefix convention is already established in the substrate.

Probe 5 (`probe_prelude_prefix_terminates_at_first_expression`) verifies the prefix-termination semantic directly: one leading define lifts; the body expression after it terminates the prefix. The child exits 0 because the lifted define is available via the prologue.

---

## Prologue ordering analysis

Prologue assembly order after Gap H:

1. **Type defs** (F-3's `topo_sort_types` over `state.captured_types`) — parent's full user type registry as AST forms
2. **Captured-binding defines** — `(:wat::core::def :__captured_X <encoded>)` for each closed-env capture
3. **User dep defines** — `topo_sort_deps` over `state.captured_deps` (functions from parent's symbol table referenced by the fn)
4. **Lifted body prelude forms** (Gap H) — define/struct/enum forms from the fn body's leading do-prefix, IN ORDER as they appeared in the source

The child's `startup_from_forms` processes forms in prologue order through its pipeline:
- Step 5 (`register_types`): processes ALL type-declaration forms in one pass — both F-3's type forms (items 1) and any `struct`/`enum` forms from the lifted prelude (items 4). Prologue ordering within step 5 affects registration order, but `TypeEnv::register` is idempotent for byte-equivalent re-registration, so out-of-order type references are handled gracefully.
- Step 6 (`register_defines`): processes `define` forms from both the dep defines (items 3) and lifted prelude defines (items 4). Order matters for mutual recursion, but the prologue's dep-define topological sort (items 3) covers parent-declared deps; the lifted prelude defines are local to the fn body and typically don't mutually recurse with parent deps.

**F-3 interaction**: steps 1 and 4 are DISTINCT sweeps in the prologue — F-3's types come from `state.captured_types` (BTreeMap, alphabetical by topo sort), Gap H's prelude forms come directly from the fn body AST in source order. They are not interleaved. The child sees parent types first, then lifted body types — which is correct (a lifted body struct may reference a parent type in its field declarations; the parent type is already registered before the body struct's `startup_from_forms` step 5 processes it).

---

## F-3 sweep interaction

F-3's type-registry sweep (lines 267-271 in `extract_closure`) populates `state.captured_types` and runs to fixpoint before the prelude-lift. The lift adds raw AST forms directly to `prologue` at step 4 — it does NOT go through `state.captured_types` or `record_type_dependency`. This is intentional:

- F-3's types are PARENT-DECLARED types serialized via `type_def_to_ast` and included at step 1.
- Gap H's lifted forms are RAW AST from the fn body — they are child-local declarations. They are not in the parent's TypeEnv and cannot be looked up via `state.parent_types.get(...)`. Including them as raw AST is the correct path; `startup_from_forms` at step 5 parses and registers them from scratch in the child's fresh TypeEnv.

**Shared helpers**: none. The two sweeps are genuinely distinct:
- F-3: `record_type_dependency` → `topo_sort_types` → `type_def_to_ast` (parent TypeDef → AST serialization)
- Gap H: `split_body_prelude` → direct prologue append (raw AST, already in correct source form)

Both sweeps coexist without conflict. F-3 probes confirm no regression.

---

## Body-shape edge case coverage

| Shape | Handled | Notes |
|-------|---------|-------|
| `(:wat::core::do define-form... expr...)` | YES | Primary case; `split_body_prelude` extracts prefix |
| `(:wat::core::do struct-form... expr...)` | YES | `is_prelude_form` matches `:wat::core::struct` |
| `(:wat::core::do enum-form... expr...)` | YES | `is_prelude_form` matches `:wat::core::enum` |
| Mixed prefix (struct + enum + define) | YES | Probe 4 verifies; all lift in source order |
| Single expression body (not a do) | NO LIFT | `split_body_prelude` returns `(vec![], body)` for non-do shapes |
| `(:wat::core::let [...] ...)` body | NO LIFT | Not a do form; no lift |
| Do body with NO leading prelude forms | NO LIFT | `prefix_len == 0` → returns `(vec![], body)` unchanged |
| Do body where ALL forms are prelude | LIFT ALL | Residual = 0 children → residual body = `:wat::core::nil` keyword |
| Do body, define AFTER expression | NOT LIFTED | Prefix-termination stops at expression; late define stays in residual body |
| Nested fn body (fn inside fn) | NO LIFT | `split_body_prelude` only operates on the TOP-LEVEL fn body passed to `extract_closure`; the inner fn's body is not split |

**Captured-local references in prelude forms**: the capture rewrite runs on `func.body` (the full body including prelude forms) BEFORE `split_body_prelude` operates. So a define like `(:wat::core::define (:h::foo -> :wat::core::i64) captured-var)` would have `captured-var` rewritten to `:__captured_captured-var` before the define is lifted. The `:__captured_captured-var` binding is in the prologue at step 2 (before the lifted define at step 4), so it resolves correctly in the child.

**Let-body containing defines**: a `let` form containing a `define` (e.g., `(:wat::core::let [x ...] (:wat::core::define ...))`) is not a do-prefix prelude. `split_body_prelude` only inspects the top-level do's children. The define inside the let body is NOT lifted and would still hit `DefineInExpressionPosition` at runtime — this is out of scope for Gap H (the A-wide doctrine lifts do-prefix forms only).

---

## Honest deltas (≥ 3)

### Delta 1 — `walk_struct_form` + `walk_enum_form` required companions (BRIEF omitted this)

The BRIEF specified only `is_fn_body_do` + `split_prelude_prefix` as helpers. The implementation revealed a third gap: `walk_free_symbols`'s List arm does NOT have handlers for `:wat::core::struct` or `:wat::core::enum`. When a struct/enum form appears in the fn body's do-prefix, the existing plain-list recursive path walks its children — including field names (bare Symbols like `x`, `value`, `field`) — and misclassifies them as free-symbol references. The extraction then fails with `UnresolvedSymbol` at `freeze_ok` time, before the child ever runs.

Fix: add `walk_struct_form` and `walk_enum_form` dispatched from `walk_free_symbols`'s List arm (same dispatch table as `let`, `fn`, `define`, `match`). These functions skip field/variant name Symbols (they are binding positions) and walk only type keyword children for type deps.

This companion fix is strictly necessary for struct/enum probes to pass. Without it, probes 2, 3, and 4 fail at `launch should evaluate` time with an `UnresolvedSymbol` error for the field name.

### Delta 2 — Prologue ordering: captured_bindings BEFORE lifted prelude forms (confirmed correct)

The BRIEF asked to verify whether order matters in the prologue (topological needs). Analysis:

Captured-binding defines (step 2, `(:wat::core::def :__captured_X ...)`) must precede the lifted prelude forms (step 4) because a prelude define might reference a closed-env capture. The capture rewrite turns `captured-var` into `:__captured_captured-var` in the lifted define body — and the `:__captured_captured-var` binding must be in the child's world before the lifted define is processed at step 6.

The current prologue ordering (steps 1–2–3–4) satisfies this: captured_bindings land at step 2, lifted prelude forms at step 4. Reordering 4 before 2 would break the `:__captured_X` reference in lifted defines.

### Delta 3 — `split_body_prelude` consumes the rewritten_body by value

The BRIEF described the helpers as taking `&[WatAST]` slices. The implementation takes `WatAST` by value for `split_body_prelude` because:
1. The rewritten body is not used after the split — the caller uses either `body_prelude_forms` or `final_body`, never both simultaneously (in the inline-lambda path, `final_body` is the fn's body; in the keyword-path path, `final_body` is the entry define's body).
2. Constructing the residual do-wrapper requires creating a new `Vec<WatAST>` anyway; owning the input avoids a clone of every child.

The `is_prelude_form` helper takes `&WatAST` for the prefix scan (read-only).

### Delta 4 — `walk_free_symbols` for struct/enum now records field type keywords as type deps

The `walk_struct_form` and `walk_enum_form` functions walk field/variant type keywords through `walk_free_symbols`. This means if a struct's field type references a PARENT-declared user type (e.g., `(value :my::ParentType)`), that type is captured into `state.captured_types` and ends up in the closure's prologue at step 1. This is strictly correct: the child needs the parent type to be registered before it can interpret the field type of the locally-declared struct.

This was not mentioned in the BRIEF but follows logically from Gap F-3's design (whole-registry sweep for exactly this reason). The `walk_struct_form`/`walk_enum_form` path is a static-reference complement to F-3's whole-registry sweep for locally-declared types whose fields reference parent types.

### Delta 5 — Probe 5 design: prefix-termination without triggering late-define error

The BRIEF's probe 5 sketch suggested testing that "defines AFTER first expression don't lift." To make the probe self-contained and not trigger a second `DefineInExpressionPosition` from a late define, probe 5 uses a do body that ends with a `let` expression (not a second define). The prefix define DOES lift; the let expression terminates the prefix. The probe verifies the lifted define is usable (call succeeds) and the child exits 0.

A probe that places a second define AFTER an expression and expects it to fail with `DefineInExpressionPosition` would require catching the child's exit code != 0, which is already proven by the pre-Gap-H baseline failures. Probe 5 instead demonstrates the positive case: that the prefix-termination correctly identifies where to stop lifting.

---

## Implementation decisions (four questions at each)

### Decision 1: `split_body_prelude` vs separate `is_fn_body_do` + `split_prelude_prefix` helpers

**BRIEF specified**: two separate helpers. **Implemented**: one combined `split_body_prelude` function returning `(Vec<WatAST>, WatAST)`.

- **Obvious**: a single entry point avoids the caller having to chain two calls and handle the `Option` from `is_fn_body_do` separately.
- **Simple**: one function, one return value, one match on the result at the call site.
- **Honest**: the BRIEF's two-helper sketch was a design guide, not a contract. The combined form is strictly equivalent.
- **Good UX**: the call site (`let (body_prelude_forms, final_body) = split_body_prelude(rewritten_body)`) is a single clear destructuring.

### Decision 2: where to apply `split_body_prelude` (before or after `rewrite_captures`)

Applied AFTER `rewrite_captures`. Rationale: lifted prelude forms may reference closed-env captures (bare Symbols in the source). The rewrite substitutes those to `:__captured_X` keywords. If we split BEFORE rewrite, the lifted forms go into the prologue with unrewritten Symbol references — which would not resolve in the child (the child has `:__captured_X` bindings, not the original bare-symbol bindings). Applying capture rewrite to the full body first (including prelude forms), then splitting, ensures lifted forms carry the correct rewritten references.

### Decision 3: residual body shape when all prelude forms are lifted

When `do_children` consists entirely of prelude forms (prefix_len == do_children.len()), the residual has 0 children. The fn body becomes `:wat::core::nil`. This is correct: the fn declares types/helpers via the prelude and has no expression to evaluate — its return value is nil. The declared return type `-> :wat::core::nil` is consistent.

---

## Verification commands run

```
# Gap H probes
cargo test --release --test probe_closure_body_prelude_lift
# → 5 passed; 0 failed

# F-3 regression
cargo test --release --test probe_spawn_process_parent_type
# → 3 passed; 0 failed

# All prior substrate probes
cargo test --release --test probe_do_splice_def --test probe_let_splice_def \
    --test probe_do_splice_define --test probe_let_splice_define \
    --test probe_do_splice_struct --test probe_do_splice_enum \
    --test probe_let_splice_struct --test probe_let_splice_enum \
    --test probe_spawn_process_parent_type \
    --test probe_resolver_quote_awareness \
    --test probe_deftest_hermetic_isolation
# → all pass

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | \
    awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# → passed:2232 failed:0
```

---

## Cross-references

- `021884a` Gap G (the blockage that revealed this gap; `DefineInExpressionPosition` analysis)
- `fe06bb1` Gap F-3 (extract_closure extension precedent; F-3's type-registry sweep untouched)
- `f9c8aef` Gap F-1 (struct/enum pre-registration in top-level do/let — parallel concern at parent scope)
- `662f5bc` Gap F-2 (resolver quote-awareness — composes with this fix's prelude-lift)
- After Gap H: `deftest-hermetic` Path E macro shape rewrite becomes actionable — the do-prefix defines in the prelude now lift to the child's prologue. See `wat/test.wat` lines 332–343 blocking comment.
- `src/closure_extract.rs` — modified (four additions)
- `tests/probe_closure_body_prelude_lift.rs` — new probe file (5 probes)
