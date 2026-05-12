# Arc 170 slice 3 Gap E BRIEF — `preregister_fn_defs_in_do`/`_in_let` must recognize `define` forms

**Sonnet.** Substrate fix that unblocks Phase E V4 (deftest rewrite). Mirror of Gap C V2 + Gap D, but for the legacy `:wat::core::define` form (vs the new `:wat::core::def`/`:wat::core::defn` family).

Phase E V3 attempted the deftest rewrite, hit this gap, and reverted to baseline cleanly. The SCORE at `SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` has the full root-cause analysis.

## Backstory — what Gap C V2 + Gap D shipped + the cascade

- **Gap C V2 (`e35b446`)**: extended `register_defines` + `register_stdlib_defines` to recurse into top-level `(:wat::core::do ...)`; added helper `preregister_fn_defs_in_do` (runtime.rs:2246).
- **Gap D (`9673721`)**: same for `(:wat::core::let ...)`; added helper `preregister_fn_defs_in_let` (runtime.rs:2293).
- **Phase E V3 attempt**: deftest expansion emits `(:wat::core::define (~name -> :wat::kernel::RunResult) (:wat::test::run-hermetic ~body))` inside a top-level `do` wrapper. **263 failures.**

Both helpers only handle `try_parse_fn_shape_def` (recognizes `(:wat::core::def :name (:wat::core::fn ...))` shapes — covers `def` and `defn`). Neither handles `is_define_form` (recognizes `(:wat::core::define (:name -> :type) body)` — the legacy form deftest still emits).

The Gap C V2 probes used `defn` (which expands to `def`), so they passed. The deftest target shape uses `define`, which the helpers don't recognize. The gap was hidden by probe-shape choice.

## Goal — extend BOTH helpers (atomic mirror)

### File: `src/runtime.rs`

**Helper 1**: `preregister_fn_defs_in_do` (current ~lines 2246-2275). Currently only calls `try_parse_fn_shape_def`. Add an `is_define_form` arm.

**Helper 2**: `preregister_fn_defs_in_let` (current ~lines 2293-2325). Identical pattern, same fix.

### Pattern (per SCORE Gap E sketch)

```rust
// Existing (lines ~2254 in preregister_fn_defs_in_do):
if let Some((path, func)) = try_parse_fn_shape_def(child) {
    if check_reserved_prefix && crate::resolve::is_reserved_prefix(&path) {
        let span = child.span().clone();
        return Err(RuntimeError::ReservedPrefix(path, span));
    }
    if !sym.functions.contains_key(&path) {
        sym.functions.insert(path, func);
    }
}

// Add (after the try_parse_fn_shape_def arm; before the nested-do recursion):
else if is_define_form(child) {
    let (path, func) = parse_define_form(child.clone())?;
    if check_reserved_prefix && crate::resolve::is_reserved_prefix(&path) {
        let span = child.span().clone();
        return Err(RuntimeError::ReservedPrefix(path, span));
    }
    if !sym.functions.contains_key(&path) {
        sym.functions.insert(path, func);
    }
}
```

Mirror the same arm into `preregister_fn_defs_in_let`. Both `parse_define_form` and `is_define_form` already exist in `src/runtime.rs` — they're used by `register_defines` directly.

`parse_define_form` takes ownership (`form: WatAST`), so `.clone()` is needed since the child stays in the `do`/`let` form in `rest`. The clone is correct + minimal.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md`** — full root-cause analysis + Gap E sketch
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`** (commit `e35b446`) — the predecessor that added the do recursion + helper
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-D-LET-SPLICE-DEF.md`** (commit `9673721`) — predecessor let mirror
4. **`src/runtime.rs:2246-2330`** — current shape of both helpers
5. **`src/runtime.rs` `is_define_form` / `parse_define_form`** — grep for current implementations (already used by `register_defines`)
6. **`tests/probe_do_splice_def.rs`** — existing regression set; new probes mirror this shape

## Implementation path

### Phase 1 — Add 2 new probes (failing baseline)

Create `tests/probe_do_splice_define.rs` with two probes:

```rust
#[test]
fn probe_do_define_two_vars_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::define (:my::helper -> :wat::core::i64)
            42)
          (:wat::core::define (:my::main -> :wat::core::i64)
            (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}

#[test]
fn probe_do_define_via_macro_emission() {
    let src = r#"
        (:wat::core::defmacro
          (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::do
             (:wat::core::define (:my::probe::helper -> :wat::core::i64)
               42)
             ~body))
        (:my::probe
          (:wat::core::define (:my::probe::main -> :wat::core::i64)
            (:my::probe::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::probe::helper").is_some());
    assert!(world.symbols().get(":my::probe::main").is_some());
}
```

Run the probes; confirm they FAIL with the resolve-time error pattern (mirrors Phase E V3 failure mode). This is the regression set.

Create `tests/probe_let_splice_define.rs` with the same shape but wrapping `(:wat::core::let [] ...)` instead of `(:wat::core::do ...)`. Confirm failure baseline.

### Phase 2 — Extend `preregister_fn_defs_in_do`

Add the `is_define_form` arm per the pattern above. Run probe — expected PASS.

### Phase 3 — Mirror into `preregister_fn_defs_in_let`

Same arm. Run let probe — expected PASS.

### Phase 4 — Verify

```bash
# Both probe sets pass
cargo test --release --test probe_do_splice_define 2>&1 | tail -5
cargo test --release --test probe_let_splice_define 2>&1 | tail -5
# Expected: 2 + 2 = 4 passed

# Existing probes still pass (Gap C V2 + Gap D regression check)
cargo test --release --test probe_do_splice_def 2>&1 | tail -5
cargo test --release --test probe_let_splice_def 2>&1 | tail -5
# Expected: 3 + 3 = 6 passed

# Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205+4 = 2209 passed / 0 failed
```

## Scope (what's IN)

- Two ~10-LOC additions to `src/runtime.rs` (one per helper)
- Two new probe test files (4 total probes — 2 do + 2 let)
- Workspace at 2209 / 0 failed (baseline + 4 probes)

## Scope (what's OUT)

- Phase E V4 deftest rewrite — separate slice (this BRIEF unblocks it)
- Closure-sync work like Gap D did — NOT NEEDED here because `define` forms don't capture let-local bindings the way fn bodies do; `define` is always at top level (verify in Phase 4)
- Anything that changes `register_defines`/`register_stdlib_defines` themselves (Gap C V2 already did those)
- `try_parse_fn_shape_def` or `parse_define_form` themselves — unchanged; only call-site addition
- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system
- New substrate features beyond the two arms

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `preregister_fn_defs_in_do` has `is_define_form` arm | grep + read |
| B | `preregister_fn_defs_in_let` has `is_define_form` arm | grep + read |
| C | `tests/probe_do_splice_define.rs` — 2 probes pass | cargo test |
| D | `tests/probe_let_splice_define.rs` — 2 probes pass | cargo test |
| E | `tests/probe_do_splice_def.rs` + `tests/probe_let_splice_def.rs` STILL pass (no regression) | cargo test |
| F | Workspace at 2209 / 0 failed (2205 baseline + 4 new probes) | full cargo test |

**6 rows.** All must PASS.

## Predicted runtime

**15-30 min sonnet.** Tight mirror of Gap C V2 + Gap D shapes; same playbook; same helper-function shape. The probes are direct adaptations of existing probe files.

**Hard cap:** 60 min (2×).

## Constraints (hard)

- DO NOT modify `is_define_form` or `parse_define_form` (only add call-site arms)
- DO NOT modify `try_parse_fn_shape_def` (existing fn-shape recognizer untouched)
- DO NOT modify `register_defines` / `register_stdlib_defines` (Gap C V2 territory)
- DO NOT modify any test call site outside the 2 new probe files
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO NOT add new substrate features
- Workspace must stay at 0 failed (baseline + new probes pass = 2209)

## Honest delta categories (anticipated)

1. **Define-form recognition order** — should the new `is_define_form` arm fire BEFORE or AFTER `try_parse_fn_shape_def`? Per arc 157, def is the new canonical and define is legacy; ordering should reflect that. Surface choice.
2. **Closure-sync needed?** — Gap D needed a closure-sync fix in `register_runtime_defs_form` because let-body fns capture let-local bindings. `define` forms don't have this issue (always top-level, no closure capture from outer scope) — verify and surface.
3. **Probe shape parity with Gap C V2 probes** — surface any deviation
4. **Anything pre-existing source-level use** — should be none; workspace was green at 2205 before the spawn
5. **Performance impact** — adding one more `else if` per child in two helpers; trivial; surface no-op

## Cross-references

- `e35b446` — Gap C V2 (the do-recursion predecessor)
- `9673721` — Gap D (the let-recursion predecessor)
- `SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` — the root-cause analysis that surfaced this gap
- arc 157 (def form) + arc 166 (defn form) — canonical replacements for legacy `define`
- arc 109 § L (task #253) — workspace `define` → `defn` rename queued separately
