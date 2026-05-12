# Arc 170 slice 3 Gap C V2 — SCORE (top-level `do` splice for `def`/`defn`)

**Scored:** 2026-05-11. Sonnet one-spawn.

## Scorecard (6 rows)

| Row | What | Result |
|-----|------|--------|
| A | All top-level form-consuming passes identified + extended | PASS — inventory below |
| B | Probe 1 (`do` of two `def` forms) passes | PASS |
| C | Probe 2 (`do` of two `defn` forms via expansion) passes | PASS |
| D | Probe 3 (defmacro-emitted `do` wrapping `defn`) passes | PASS |
| E | Workspace at 0 failed | PASS — 2202 passed / 0 failed |
| F | `cargo check --release` green | PASS — clean, 0 errors |

**All 6 rows PASS.**

## Workspace delta

- Baseline: 2199 passed / 0 failed
- Post Gap C V2: 2202 passed / 0 failed (+3 probes, all new, all passing)

## Files changed

| File | Change |
|------|--------|
| `src/runtime.rs` | `register_defines` + `register_stdlib_defines` extended with do arm; `preregister_fn_defs_in_do` helper added |
| `tests/probe_do_splice_def.rs` | Three regression probes added (new file) |

## Complete pass inventory

### Passes that ALREADY handled `(:wat::core::do ...)` (not changed)

| Pass | Location | Do arm present since |
|------|----------|----------------------|
| `register_runtime_defs` | `src/runtime.rs:2018` | Arc 136 (runtime eval) |
| `collect_splice_defs_ctx` | `src/check.rs:6848` | Arc 157 (def legality) |

### Passes that DO NOT need a do arm (scope analysis)

| Pass | Location | Why no do arm needed |
|------|----------|----------------------|
| `register_defmacros` / `register_stdlib_defmacros` | `src/macros.rs:269, 290` | Looks for `defmacro` only; `defmacro` inside `do` is a separate, not-yet-demanded concern |
| `register_types` / `register_stdlib_types` | `src/types.rs:1182, 1209` | Looks for type declarations (`struct`/`enum`/`newtype`/`typealias`); type decls inside `do` are a separate concern |
| `register_define_dispatches` / `register_stdlib_define_dispatches` | `src/dispatch.rs:247, 264` | Looks for `define-dispatch` only; dispatch decls inside `do` are a separate concern |
| `resolve_references` | `src/resolve.rs:84` | Does not register; validates call heads. Already recurses into all children via `check_form`. Gap was upstream in `register_defines` not populating `sym.functions`. |
| `check_program` | `src/check.rs:1548` | Handles do via `collect_splice_defs_ctx` (arc 157); the type-check path already sees-through top-level `do` |
| `register_struct_methods` / `register_enum_methods` / `register_newtype_methods` | `src/runtime.rs:1604, 1723, 1839` | Iterate `TypeEnv`, not AST forms; no do arm needed |

### Pass extended in Gap C V2

| Pass | Location | What was added |
|------|----------|----------------|
| `register_defines` | `src/runtime.rs:1492-1507` | Do arm: when a top-level form is `(:wat::core::do ...)`, call `preregister_fn_defs_in_do` to peek inside and pre-register any fn-shape defs into `sym.functions`. The `do` form itself stays in `rest`. |
| `register_stdlib_defines` | `src/runtime.rs:1534-1547` | Mirror of the above; bypasses reserved-prefix check (stdlib is privileged). |

### New helper

**`preregister_fn_defs_in_do`** (`src/runtime.rs:2215-2244`)

Scans the children of a `do` form and calls `try_parse_fn_shape_def` on each child. Pre-registers any fn-shape defs found into `sym.functions`. Recurses into nested `do` forms (handles macro-emitted nested dos). Takes `check_reserved_prefix: bool` to distinguish user source (true) from stdlib source (false).

## Resolve pass mechanism (most subtle)

The resolve pass (`resolve_references`, `src/resolve.rs:84`) already recursed into all children of every list form via `check_form` → `for child in items { check_form(child, ...) }`. This means it correctly found calls to `:my::helper` inside the `do` body. The gap was NOT in the resolve pass itself.

The gap was one step earlier: `register_defines` (step 6 in the startup pipeline) did not recurse into `(:wat::core::do ...)` to pre-register fn-shape `def` forms into `sym.functions`. So when resolve ran at step 7, `sym.functions` was empty of those names, and `is_resolvable_call_head` returned false for them.

Fix: `register_defines` now peeks into top-level `do` children via `preregister_fn_defs_in_do`, inserting fn-shape defs into `sym.functions` before resolve runs. The `do` form itself remains in `rest` and is later processed by `register_runtime_defs` (which already had the do arm).

The resolve pass itself required no changes.

## Top-level `let` parallel gap (Phase 5)

**Gap exists.** `register_defines` also does not recurse into `(:wat::core::let ...)` body forms to pre-register fn-shape defs. The same resolve-time failure would occur with:

```wat
(:wat::core::let [x 1]
  (:wat::core::def :my::f (:wat::core::fn [] -> :wat::core::i64 x))
  (:wat::core::def :my::g (:wat::core::fn [] -> :wat::core::i64 (:my::f))))
```

`register_runtime_defs` already handles `let` with a `let` arm (src/runtime.rs:2024). `collect_splice_defs_ctx` already handles `let` with a `let` arm (src/check.rs:6853). `register_defines` does not.

This is Gap D. It is NOT fixed in this slice per the BRIEF scope constraint. A targeted follow-up arc mirrors this slice's `preregister_fn_defs_in_do` pattern for `let`.

## Honest deltas

1. **The resolve pass needed no changes.** The failure was not IN the resolver — it already recursed into all children. The gap was that `register_defines` ran before `resolve_references` and didn't pre-register fn-shape defs found inside top-level `do`. One targeted fix in one function (with a stdlib mirror) closed the gap.

2. **`register_stdlib_defines` also lacked the do arm.** The BRIEF listed `register_defines` as the primary candidate; `register_stdlib_defines` is its privileged mirror and was extended simultaneously. No stdlib sources currently exercise this path, but consistency requires the mirror.

3. **Nested `do` handled proactively.** The helper `preregister_fn_defs_in_do` recurses into nested `do` forms. The three probes don't exercise nested dos, but a macro that emits `(do (do defn-a) defn-b)` would have the same gap without the recursive descent. Probe 3's macro emission path goes one level deep and passes.

4. **Let-gap (Gap D) confirmed present, not fixed.** `register_runtime_defs` and `collect_splice_defs_ctx` both have `let` arms. `register_defines` does not. The same resolve-time failure occurs for fn-shape defs inside a top-level `let` body that cross-call each other. Surfaced per Phase 5 instruction; fix is a separate arc.

5. **Workspace impact: 0 existing tests changed.** The 2199 pre-existing tests all continued to pass. No pre-existing test was relying on the gap (e.g., no test expected resolve to fail on do-nested defs). The 3 new probes are the only delta.

## Cross-references

- Arc 136 (do form): `docs/arc/2026/05/136-core-do-form/INSCRIPTION.md`
- Arc 157 (def form): `docs/arc/2026/05/157-core-def-form/INSCRIPTION.md`
- Arc 166 (def-of-fn pre-registration): arc 166 established `try_parse_fn_shape_def` and the pre-registration discipline this slice extends
- Gap D follow-up: `register_defines` + `let` body — same pattern as Gap C but for `let` splice position
- Phase E V3 (next): `deftest` macro emits `(:wat::core::do ~@prelude (:wat::core::defn ~name ...))` — top-level `do` now splices uniformly; prelude defines + test fn all register
