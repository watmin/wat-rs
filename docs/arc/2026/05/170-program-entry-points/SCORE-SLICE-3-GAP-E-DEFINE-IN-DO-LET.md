# Arc 170 slice 3 Gap E — SCORE (`define` forms in `preregister_fn_defs_in_do` + `_in_let`)

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2209 passed / 0 failed

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `preregister_fn_defs_in_do` (runtime.rs ~2263) has `is_define_form` arm | grep + read | PASS — `else if is_define_form(child)` arm present between `try_parse_fn_shape_def` arm and nested-do recursion |
| B | `preregister_fn_defs_in_let` (runtime.rs ~2322) has `is_define_form` arm | grep + read | PASS — identical arm present; same position relative to `try_parse_fn_shape_def` and nested-let recursion |
| C | `tests/probe_do_splice_define.rs` — 2 probes pass | cargo test | PASS — 2 passed / 0 failed |
| D | `tests/probe_let_splice_define.rs` — 2 probes pass | cargo test | PASS — 2 passed / 0 failed |
| E | `tests/probe_do_splice_def.rs` + `tests/probe_let_splice_def.rs` still pass (no regression) | cargo test | PASS — 3 + 3 = 6 passed / 0 failed |
| F | Workspace at 2209 / 0 failed (2205 baseline + 4 new probes) | full cargo test | PASS — `passed:2209 failed:0` |

**All 6 rows PASS.**

---

## Files changed

| File | Change |
|------|--------|
| `src/runtime.rs` | `preregister_fn_defs_in_do`: added `is_define_form` arm (~10 LOC). `preregister_fn_defs_in_let`: mirror arm (~10 LOC). No other changes. |
| `tests/probe_do_splice_define.rs` | 2 new regression probes (new file). |
| `tests/probe_let_splice_define.rs` | 2 new regression probes (new file). |

---

## Workspace delta

- Baseline: 2205 passed / 0 failed
- Post Gap E: 2209 passed / 0 failed (+4 probes, all new, all passing)

---

## Arm-order rationale: `is_define_form` AFTER `try_parse_fn_shape_def`

The `else if is_define_form` arm fires AFTER the `try_parse_fn_shape_def` arm in both helpers. Rationale (four questions):

- **Obvious**: arc 157 established `def` as the new canonical form; arc 166 established `defn` as the ergonomic surface. Both expand to `def`-of-fn. `define` is the legacy form. Canonical checks before legacy checks is the obvious order.
- **Simple**: the order is consistent with `register_defines` itself, which checks `is_define_form` at the top but that is because `define` IS the primary consumed form at the top level — it is consumed (not kept in `rest`). In the helpers, `def`-of-fn is the new canonical pre-registration target; `define` is the legacy fallback.
- **Honest**: a form cannot be both a `def`-of-fn shape AND a `define` form (different head keywords: `:wat::core::def` vs `:wat::core::define`). The order is therefore irrelevant for correctness — neither branch can shadow the other. The canonical-before-legacy ordering is a documentation/intent signal, not a correctness constraint.
- **Good UX**: placing `is_define_form` BEFORE the nested-do/let recursion is correct — a define form is a leaf, not a container, so no recursive descent applies to it.

---

## Closure-sync verification: define-in-let does NOT need the Gap D fix

Gap D's unexpected complication: `preregister_fn_defs_in_let` inserts a stub (`closed_env: None`) into `sym.functions`. `eval_tail` dispatches through `sym.functions` first. The stub won over the correctly-closed fn in `runtime_def_values`. Fix: `register_runtime_defs_form`'s `def` arm now writes the evaluated fn BACK into `sym.functions`, overwriting the stub with the properly-closed fn.

For `define`-in-let, this issue does NOT arise. Here is why:

1. `register_defines` consumes `define` forms DIRECTLY — they are extracted from the top-level form list and inserted into `sym.functions`; they are NOT kept in `rest`. A `define` inside a `do` or `let` body stays in `rest` as part of the `do`/`let` form, which IS kept in `rest`. But the pre-registered entry (`closed_env: None`) is inserted by `preregister_fn_defs_in_do/let` at pre-registration time.

2. When `register_runtime_defs` runs later on the `do`/`let` form in `rest`, it calls `register_runtime_defs_form` on each child. For a `define` child, `register_runtime_defs_form` hits the `_ =>` wildcard arm (line 2131) and does nothing. There is no runtime evaluation of the define form through this path.

3. `define` functions do NOT close over let-local bindings. The `define` form's body is evaluated at CALL time via `eval` (not at freeze time). The body references are resolved from `sym.functions` + runtime context at the call site, not from the enclosing let scope at freeze time. `closed_env: None` is the correct and complete representation for a define-registered function.

4. Confirmed via test: `probe_let_define_two_vars_visible` passes without any closure-sync work. A `define` inside a `let []` body calls another `define` inside the same body; the pre-registered stub is sufficient for resolve-time validation AND call dispatch.

**Conclusion: closure-sync is a no-op for define-in-let. The Gap D fix is specific to `def`-of-fn where the fn body captures let-local bindings via `closed_env` at freeze time.**

---

## Honest deltas

### Delta 1 — Both form types share `closed_env: None`; the structural distinction is what dictates behavior

`try_parse_fn_shape_def` produces a `Function` with `closed_env: None`. `parse_define_form` also produces a `Function` with `closed_env: None`. Both stubs serve the same resolve-time pre-registration purpose. The distinction: `def`-of-fn forms are kept in `rest` and evaluated at freeze time (potentially closing over let-locals via `register_runtime_defs_form`'s let arm); `define` forms inside a do/let are pre-registered but their runtime evaluation is deferred to call time (not freeze time). The Gap D closure-sync fix targets the first path exclusively.

### Delta 2 — The probe shape mirrors the SCORE-E sketch accurately

The SCORE for Phase E V3 (`SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md`) contained a sketch probe `probe_do_define_visible`. The BRIEF formalized it as `probe_do_define_two_vars_visible` and added `probe_do_define_via_macro_emission` (the Phase E use case directly). The let probes were added for symmetry per Gap D precedent. The final probes match the BRIEF exactly.

### Delta 3 — The `_ =>` arm in `register_runtime_defs_form` is the authoritative documentation that define is call-time, not freeze-time

Line 2131-2134 in `register_runtime_defs_form`:
```rust
_ => {
    // Non-splice top-level form (define, struct, enum, etc.) —
    // not a def-eligible position. No action needed.
}
```
This comment names `define` explicitly. The architecture is intentional: `register_defines` owns the freeze-time registration of define forms (consuming them from the form list); `register_runtime_defs_form` explicitly excludes them. Gap E correctly follows this architecture: pre-registration is done by the helpers (inserting into `sym.functions`); no runtime re-evaluation is attempted.

### Delta 4 — Probe 2 (macro-emission) is the Phase E V4 unblock proof

`probe_do_define_via_macro_emission` directly exercises the deftest expansion path: a macro emits `(:wat::core::do prelude-form (:wat::core::define (name -> type) body))` at top level. Both the prelude define and the body define pre-register. Phase E V4 (the deftest rewrite) can now proceed with the confidence that the substrate gap is closed.

### Delta 5 — Performance: one additional `else if` per child in two helpers; trivial

Each child in a `do`/`let` body now hits at most three checks: `try_parse_fn_shape_def` (cheap: len check + keyword check), `is_define_form` (cheap: single keyword match), and the nested-do/let recursion (structural `matches!`). For non-define, non-def, non-container children, all three checks fire and short-circuit. The overhead is negligible — these helpers run once per file at freeze time.

---

## Cross-references

- `e35b446` — Gap C V2 (`preregister_fn_defs_in_do` added, handles `def`/`defn`)
- `9673721` — Gap D (`preregister_fn_defs_in_let` added, handles `def`/`defn`; closure-sync fix)
- `SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` — root-cause analysis that surfaced this gap + Gap E sketch
- `SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md` — the do-recursion predecessor
- `SCORE-SLICE-3-GAP-D-LET-SPLICE-DEF.md` — the let-recursion predecessor
- Arc 157 (def form): canonical replacement for legacy `define`
- Arc 166 (defn form): ergonomic surface; expands to `def`-of-fn
- Arc 109 § L (task #253): workspace `define` → `defn` rename queued separately
- Phase E V4 (next): deftest macro rewrite — now unblocked
