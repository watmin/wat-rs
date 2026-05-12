# Arc 170 slice 3 Gap F-1 BRIEF — pre-generate struct/enum accessors in top-level `do`/`let` splice

**Sonnet.** Fourth iteration of the `preregister_fn_defs_in_do`/`_in_let` extension pattern. Pattern-mirrors Gap C V2 (`e35b446`), Gap D (`9673721`), and Gap E (`3d65b82`). The substrate gap discovered by Phase E V4 (commit `f2de549` SCORE).

Surfaced by V4 failure pattern 1: helper `define` forms inside top-level `do` blocks that call struct/enum accessors (`:svc::State/new`, `:svc::Request::Push`) fail outer `resolve_references` because struct/enum accessors are generated at EVAL time, not at `register_defines` time.

## Goal — extend BOTH helpers (atomic mirror of Gap E shape)

`preregister_fn_defs_in_do` (runtime.rs:2246) + `preregister_fn_defs_in_let` (runtime.rs:2293) currently recognize:
- `try_parse_fn_shape_def` — `:wat::core::def`/`:wat::core::defn` (Gap C V2)
- `is_define_form` — `:wat::core::define` (Gap E)

This Gap adds:
- `is_struct_form` — `:wat::core::struct` (pre-generate `Type/new` + field accessors stubs)
- `is_enum_form` — `:wat::core::enum` (pre-generate variant constructors `Type::Variant` stubs)

## Closure-sync requirement (mirrors Gap D's pattern)

`register_runtime_defs_form`'s struct/enum arms must — like Gap D's `def` arm — write the EVALUATED accessor functions BACK into `sym.functions`, overwriting the stubs inserted by the pre-registration helper. Confirms `sym.functions` stays authoritative for both validation and dispatch.

If struct/enum runtime registration doesn't go through `register_runtime_defs_form` (different code path), the closure-sync gap may not apply directly — verify the runtime registration path and surface the equivalent point in SCORE.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`** — the V4 failure analysis that identified Gap F
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-E-DEFINE-IN-DO-LET.md`** (commit `3d65b82`) — the immediate precedent (define-form recognition)
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-D-LET-SPLICE-DEF.md`** (commit `9673721`) — closure-sync precedent
4. **`src/runtime.rs:2246-2330`** — current shape of both helpers (after Gap E)
5. **`src/runtime.rs` `register_runtime_defs_form`** — closure-sync pattern point (Gap D fix lives here)
6. **`src/runtime.rs` / `src/types.rs`** — struct/enum runtime registration path; `is_struct_form` / `is_enum_form` predicates if they exist (otherwise mint them)
7. **`tests/probe_do_splice_define.rs`** + **`tests/probe_let_splice_define.rs`** — Gap E probe shape to mirror

## Implementation path

### Phase 1 — Identify the predicates + accessor-generation entry points

Grep for `is_struct_form`, `is_enum_form`, or their equivalents. If not present, mint them per the `is_define_form` shape:

```rust
fn is_struct_form(form: &WatAST) -> bool {
    matches!(form, WatAST::List(items, _) if matches!(
        items.first(),
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::struct"
    ))
}
```

Identify the accessor-generation entry point — the function that creates `Type/new`, field accessors, variant constructors. Likely in `src/runtime.rs` or `src/types.rs`. Sonnet picks the right pre-registration shape (full generation OR stub-style — see Phase 2).

### Phase 2 — Write probes (failing baseline)

Create `tests/probe_do_splice_struct.rs` + `tests/probe_let_splice_struct.rs` + `tests/probe_do_splice_enum.rs` + `tests/probe_let_splice_enum.rs` — 4 probe files OR 1 file with all 4. Sonnet picks the structure.

Probe shape (do + struct):

```rust
#[test]
fn probe_do_struct_accessor_visible() {
    let src = r#"
        (:wat::core::do
          (:wat::core::struct :my::State
            (counter :wat::core::i64))
          (:wat::core::define (:my::main -> :my::State)
            (:my::State/new 42)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("freeze");
    assert!(world.symbols().get(":my::State/new").is_some());
    assert!(world.symbols().get(":my::main").is_some());
}
```

Same shape for enum (constructor + matching). Mirror to `let`-body.

Confirm baseline FAILS with the resolve-time error from V4 SCORE failure pattern 1.

### Phase 3 — Extend `preregister_fn_defs_in_do`

Add `is_struct_form` arm + `is_enum_form` arm after the existing `is_define_form` arm. Each arm calls the appropriate accessor pre-generation function (or generates stubs) and inserts into `sym.functions`.

### Phase 4 — Mirror into `preregister_fn_defs_in_let`

Same arms, same shape.

### Phase 5 — Closure-sync verification (Gap D pattern)

If structs/enums go through `register_runtime_defs_form`, ensure their arms write back to `sym.functions` (mirror Gap D's `def` arm). If they go through a separate path, verify that path produces the canonical accessors and the pre-registered stubs are correctly replaced.

### Phase 6 — Verify

```bash
# Gap F-1 probes
cargo test --release --test probe_do_splice_struct
cargo test --release --test probe_let_splice_struct
cargo test --release --test probe_do_splice_enum
cargo test --release --test probe_let_splice_enum
# Expected: all probes pass

# Regression check on existing probes
cargo test --release --test probe_do_splice_def     # 3 expected
cargo test --release --test probe_let_splice_def    # 3 expected
cargo test --release --test probe_do_splice_define  # 2 expected
cargo test --release --test probe_let_splice_define # 2 expected

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2209 + N new probes (4-8 depending on probe-per-file vs combined) / 0 failed
```

## Scope (what's IN)

- Two helper functions extended (`preregister_fn_defs_in_do` + `_in_let`) with struct/enum arms
- New probes covering struct + enum in both do and let
- Closure-sync verification or extension
- Workspace stays at 0 failed (baseline + new probes)

## Scope (what's OUT)

- Phase E V5 / deftest rewrite — separate slice (this BRIEF is its prerequisite)
- Gap F-2 (resolver quote-awareness) — separate slice
- Gap F-3 (closure extraction type-registry inheritance) — separate slice
- Any change to struct/enum semantics outside the pre-registration concern
- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `preregister_fn_defs_in_do` has `is_struct_form` + `is_enum_form` arms | grep + read |
| B | `preregister_fn_defs_in_let` has matching arms | grep + read |
| C | New probes pass (4 minimum: do/let × struct/enum) | cargo test |
| D | Existing 10 Gap C V2 + D + E probes still pass (no regression) | cargo test |
| E | `cargo check --release` green; workspace at 2209 + N passed / 0 failed | full test |
| F | Closure-sync verified or extended | SCORE documents the path |

**6 rows.** All must PASS.

## Predicted runtime

**30-60 min sonnet.** Mirror of existing pattern; sonnet has 3 prior iterations of this exact shape. Closure-sync is the unknown — could be no-op (different runtime path) or mirror of Gap D (define-style closure-sync fix needed).

**Hard cap:** 120 min (2×).

## Constraints (hard)

- DO NOT modify `is_struct_form` / `is_enum_form` / `parse_struct_form` / `parse_enum_form` if they exist with different shapes (verify behavior)
- DO NOT modify `register_defines` / `register_stdlib_defines` (Gap C V2 territory; the helpers are what get extended)
- DO NOT modify any test call site outside the new probe files
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO NOT extend to Gap F-2 (resolver) or Gap F-3 (closure extraction) scope — those are separate slices

## Honest delta categories (anticipated)

1. **Pre-registration shape — stub vs full**: do we pre-generate stubs (placeholder fns with no body) for resolver to see, then real accessors at eval-time replace them (Gap D pattern)? OR do we generate full accessors at pre-registration time (cleaner but more work)? Surface the choice + rationale.
2. **Closure-sync requirement** — does it apply, and what's the fix shape if so?
3. **Probe granularity** — 4 separate files OR 1 combined? Mirror existing probe organization.
4. **Sub-form coverage** — struct with typealias, enum with parametric variant, etc. — surface any sub-form not covered.
5. **Anything unexpected** — particularly if struct/enum's existing registration path doesn't compose cleanly with pre-registration.

## Cross-references

- `e35b446` Gap C V2 (do-recursion predecessor)
- `9673721` Gap D (let-recursion + closure-sync precedent)
- `3d65b82` Gap E (define-form predecessor)
- V4 SCORE (failure analysis identifying Gap F): `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`
- Phase E V5 — unblocked after Gap F-1 + F-2 + F-3 all ship
