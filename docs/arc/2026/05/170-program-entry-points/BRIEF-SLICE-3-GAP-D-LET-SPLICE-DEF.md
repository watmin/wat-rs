# Arc 170 slice 3 — Gap D BRIEF (top-level `let` splice for `def`/`defn`)

**Sonnet.** Symmetric mirror of Gap C V2 (commit `e35b446`). Extend `register_defines` + `register_stdlib_defines` to recurse into top-level `(:wat::core::let bindings body...)` body forms — the parallel gap sonnet surfaced while shipping Gap C.

User direction 2026-05-12: *"we do not leave defects in our code."*

Per arc 157 doctrine (`src/check.rs:715`): def is legal at top level position (1) direct file top-level, (2) inside top-level `do`, **(3) inside top-level `let` body**. Position (3) is recognized by the def-legality check (`collect_splice_defs_ctx` at check.rs:6848 — handles `let`) but NOT by `register_defines` / `register_stdlib_defines`. Same shape as Gap C's gap; same shape of fix.

## Goal (precise, narrow scope)

Make these three probes pass (currently fail with the same resolve-time call-head-lookup error Gap C V2 fixed for `do`):

```rust
#[test]
fn probe_let_def_two_vars_visible() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::def :my::helper (:wat::core::fn [] -> :wat::core::i64 42))
          (:wat::core::def :my::main (:wat::core::fn [] -> :wat::core::i64 (:my::helper))))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}

#[test]
fn probe_let_defn_via_expansion() {
    let src = r#"
        (:wat::core::let []
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}

#[test]
fn probe_let_with_real_bindings_then_defn() {
    // Non-empty bindings; defns in the body. Verify bindings + def coexist.
    let src = r#"
        (:wat::core::let [x (:wat::core::i64::+'2 1 1)]
          (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
          (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::helper").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}
```

## Implementation reference — mirror Gap C V2

Gap C V2 (`e35b446`) added:
- `do` arm in `register_defines` (runtime.rs:1492-1507) — recurses into `(:wat::core::do ...)` children calling `preregister_fn_defs_in_do` helper
- `do` arm in `register_stdlib_defines` (runtime.rs:1534-1547) — same shape
- Helper `preregister_fn_defs_in_do` at runtime.rs:2215

Gap D mirrors:
- Add a `let` arm in `register_defines` that recurses into the let body (items[2..] per arc 168 multi-form body)
- Add the same arm to `register_stdlib_defines`
- Add a `preregister_fn_defs_in_let` helper (or generalize `preregister_fn_defs_in_do` to handle both forms — sonnet picks the cleaner shape)

The check pass's `collect_splice_defs_ctx` at check.rs:6853 already handles let:
```rust
":wat::core::let" if is_top => {
    // Arc 168 multi-form body: any body form may be a def position;
    // iterate all of them.
    for body_form in &items[2..] {
        collect_splice_defs_ctx(body_form, true, env, fresh, errors);
    }
}
```

Mirror this body-iteration shape in `register_defines` / `register_stdlib_defines`.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`** — Gap C V2 SCORE; the pattern this slice mirrors. Sonnet surfaced "Gap D — let parallel gap" in its findings.
2. **`src/runtime.rs:1492-1547`** — Gap C V2's `do` arms (the pattern to mirror)
3. **`src/runtime.rs:2215`** — `preregister_fn_defs_in_do` helper
4. **`src/check.rs:6853`** — existing `let` arm in `collect_splice_defs_ctx`
5. **`src/check.rs:715`** — arc 157 § Scope Q1 doctrine (def-in-top-level-let legal)
6. **`tests/probe_do_splice_def.rs`** — Gap C V2's three probes (pattern for these three)

## Implementation path

### Phase 1 — Write the three probes (failing baseline)

Create `tests/probe_let_splice_def.rs` with the three probes from above. Confirm they fail with the resolve-time error. This is the regression set.

### Phase 2 — Extend the two registration functions

Add a `let` arm to `register_defines` (after the `do` arm) and the matching arm to `register_stdlib_defines`. Each recurses into items[2..] (the body forms, per arc 168 multi-form body).

Either reuse the `preregister_fn_defs_in_do` helper if it generalizes naturally (and rename to `preregister_fn_defs_in_splice` or similar), OR add a parallel `preregister_fn_defs_in_let` helper. Sonnet picks the cleaner shape.

### Phase 3 — Verify

All three probes pass; full workspace at 0 failed.

### Phase 4 — Check parallel let* gap

`let*` is the sequential-binding sibling of `let`. Does it have the same gap? Check `collect_splice_defs_ctx` — does it have a let* arm? Surface presence/absence; do not fix unless it's the same shape AND trivial. Out of scope; track for follow-up if needed.

## Scope (what's IN)

- `register_defines` + `register_stdlib_defines` extended with `let` arms
- Helper function (new or generalized)
- Three probe tests in `tests/probe_let_splice_def.rs`
- Workspace stays at 0 failed

## Scope (what's OUT)

- `let*` parallel gap — surface only
- deftest macro rewrite (Phase E V3) — separate
- Phase F / Slice 4 / arc 109 renames — separate
- Anything else

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `register_defines` extended with `let` arm | grep |
| B | `register_stdlib_defines` extended | grep |
| C | All three probes pass | cargo test |
| D | Workspace at 0 failed | full cargo test |
| E | `cargo check --release` green | clean |
| F | SCORE documents implementation + let* gap status (surface only) | manual review |

**6 rows.**

## Predicted runtime

**30-60 min sonnet.** Smaller than Gap C V2 (45-120 min predicted, ~10 min actual). The pattern is established; the work is mechanical mirror.

**Hard cap:** 120 min.

## Constraints (hard)

- DO NOT commit
- DO NOT touch deftest / deftest-hermetic (Phase E V3)
- DO NOT modify Layer 1/2 macros / drivers
- DO NOT retire run-sandboxed-* (Phase F)
- DO NOT touch BareLegacy* / spawn.rs / Process<I,O> struct fields
- DO NOT rename define→defn workspace-wide
- DO NOT extend let* in this slice (surface only)
- DO NOT use deferral language in SCORE
- Workspace must stay at 0 failed

## Honest delta categories (anticipated)

1. **Helper generalization vs duplication** — did the do helper generalize cleanly, or separate let helper needed
2. **let* gap status** — exists / doesn't / didn't check
3. **Anything unexpected** during 3-probe verification

## Cross-references

- Gap C V2 SCORE (the precedent): [`SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`](./SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md)
- Probe pattern: `tests/probe_do_splice_def.rs`
- Arc 157 doctrine: `src/check.rs:715` (top-level let body legal for def)
- Phase E V3 (next): now unblocked across `do` AND `let` registrations
