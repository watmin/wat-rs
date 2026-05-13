# Arc 170 slice 3 Gap J BRIEF — `register_types` splice-awareness

**Sonnet.** Substrate fix that closes the gap surfaced by Phase E V5's failed attempt (`5d82e92`-baseline test → 13 failures, reverted). Extend `register_types` (`src/types.rs:1182`) to recurse into top-level `(:wat::core::do ...)` and `(:wat::core::let ...)` forms, registering type declarations (struct/enum/newtype/typealias) nested inside.

## ⚠ Latent substrate defects may exist

**Orchestrator + user direction 2026-05-14 (after a prior attempt of this BRIEF was killed mid-run):**

The substrate may have **latent defects we haven't named**. The diagnose round that produced this BRIEF was strong evidence for the typealias-in-do gap — but V5's failures may trace to MORE than one substrate gap. The previous Gap J attempt got the splice-awareness fix correct AND closed 3 of 10 V5 retry failures, but **7 remained**, and those remaining failures may indicate further latent substrate defects (turbofish generics in spawn-process child, closure-extraction prologue for complex transitive deps, sandboxed-ast vs run-hermetic semantic differences, etc.).

**Your mission has TWO acceptable success modes:**

**Mode A — Forge through cleanly:** apply the Gap J fix + V5 retry + all 13 previously-failing tests pass. Workspace at 2243 + N / 0 failed. The single-fix hypothesis holds.

**Mode B — Pinpoint latent defects:** apply the Gap J fix (which is solid based on the diagnose). If V5 retry STILL has failures, **STOP** and produce a detailed audit. For each remaining failure: minimum-repro probe, code-trace, root-cause hypothesis, and certainty rating. Do NOT work around substrate issues. Do NOT paper over. Do NOT ship a "partial fix that passes some tests but masks others." The user's foundation-priority bar means pinpointing substrate defects IS a valid mission outcome.

**Either outcome closes this task.** Mode B's findings drive the next slices (Gap K, Gap L, etc.). The substrate-as-teacher cascade depends on honest naming of what's broken, not on forcing the V5 retry to look like it passes.

Diagnose (full reasoning in `INTERSTITIAL-REALIZATIONS.md` § "V5 boss-fight + Gap J diagnosis"):

- Direct TypeEnv probe proved: ALL FOUR type-decl kinds are ABSENT from TypeEnv when nested in top-level `do`.
- Struct/enum/newtype CONSUMERS pass type-check anyway via backup paths:
  - Struct/enum: `preregister_struct_accessors_from_form` / `preregister_enum_constructors_from_form` (Gap F-1) — accessor stubs in `sym.functions`
  - Newtype: nominal opacity (no structural lookup needed)
- **Typealias has no backup.** `expand_alias(types, path)` queries TypeEnv directly. Without registration, returns the path unchanged; unification fails.
- Three V5 patterns trace to this same gap:
  - **A (typealias unification)** — directly proven
  - **B (match scrutinee = Option<?>)** — match-pattern inference needs TypeEnv for enum variant→enum binding
  - **C (child exit-3)** — Gap F-3 propagates parent's TypeEnv to spawned child; incomplete parent → incomplete child

User direction 2026-05-14 (foundation priority): *"is the path is clear - we step forward."*

## Goal

`register_types` walks top-level forms; when it encounters a `do` or `let` form, recurse into the body and register type declarations nested inside (same way `preregister_fn_defs_in_do`/`_in_let` already recurse for def/define/struct/enum accessor pregen).

After the fix:
- `(:wat::core::do (:typealias :Foo :i64) ...)` at top level → `:Foo` lands in TypeEnv
- Same for struct, enum, newtype nested in do
- Same recursion for nested do-within-do (mirror existing splice machinery)
- Equivalent for top-level `let` body (per arc 168 multi-form body, items[2..])

## Required reading IN ORDER

1. `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` § "V5 boss-fight + Gap J diagnosis" — full reasoning + diagnose trail
2. `src/types.rs` lines 1182-1240 — `register_types` + `register_stdlib_types` + `classify_type_decl` (the surface to extend)
3. `src/runtime.rs` lines 2475-2585 — `preregister_fn_defs_in_do` + `preregister_fn_defs_in_let` (the splice-recursion pattern to mirror)
4. `src/freeze.rs` lines 580-630 — the freeze pipeline call site for `register_types`
5. `tests/probe_do_splice_struct.rs` — precedent regression-probe shape for top-level do splice

## Implementation path

### Phase 1 — Mint splice-recursion helpers (15-20 min)

In `src/types.rs`, after `register_types` and `register_stdlib_types`, add:

```rust
/// Recurse into a do form's body, extracting + registering any type
/// declarations and returning the form with type-decls stripped.
fn process_do_for_types(
    items: Vec<WatAST>,
    env: &mut TypeEnv,
    span: Span,
    stdlib: bool,  // routes to env.register_with_span or env.register_stdlib_with_span
) -> Result<WatAST, TypeError> {
    // items[0] = :wat::core::do keyword
    // items[1..] = body children
    let mut new_children = Vec::with_capacity(items.len());
    new_children.push(items[0].clone());
    for child in items.into_iter().skip(1) {
        // 1. If child is a type-decl form → register, drop from new_children
        // 2. If child is a nested do/let → recurse, push reconstructed form
        // 3. Otherwise → push unchanged
        ...
    }
    Ok(WatAST::List(new_children, span))
}

/// Same for let — items[0]=:let, items[1]=bindings, items[2..]=body
fn process_let_for_types(
    items: Vec<WatAST>,
    env: &mut TypeEnv,
    span: Span,
    stdlib: bool,
) -> Result<WatAST, TypeError> {
    // items[0]/items[1] preserved; items[2..] = body forms (arc 168 multi-form body)
    // Recurse on each body form like the do case
    ...
}
```

Then in `register_types` and `register_stdlib_types`, extend the `None` arm of the `classify_type_decl(&form)` match to detect do/let forms and call the new helpers. The recursion replaces the form with its type-decl-stripped equivalent before pushing to `rest`.

### Phase 2 — Regression probes (10-15 min)

Create `tests/probe_register_types_splice_aware.rs`:

```rust
use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn do_typealias_registers_in_type_env() {
    let src = r#"
        (:wat::core::do
          (:wat::core::typealias :diag::MyAlias :wat::core::i64))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup failed");
    assert!(world.types().get(":diag::MyAlias").is_some());
}

#[test]
fn do_struct_registers_in_type_env() {
    // Mirror; verify struct TypeDef lands in env.types()
}

#[test]
fn do_newtype_registers_in_type_env() {
    // Mirror; verify newtype TypeDef lands in env.types()
}

#[test]
fn do_enum_registers_in_type_env() {
    // Mirror; verify enum TypeDef lands in env.types()
}

#[test]
fn let_body_typealias_registers() {
    // Verify let-body splice equivalent
}

#[test]
fn nested_do_typealias_registers() {
    // (do (do (typealias :A :i64))) — recursion case
}

#[test]
fn do_typealias_usage_typechecks() {
    // (do (typealias :A :i64) (define (f -> :A) 42)) — END-TO-END proof
}
```

### Phase 3 — V5 retry (THE BIG VERIFICATION) (15-20 min)

After the fix + probes pass, apply Phase E V5's deftest macro rewrite (from the V4 BRIEF target shape):

```scheme
;; In wat/test.wat — the deftest defmacro body:
`(:wat::core::do
   ~@prelude
   (:wat::core::define (~name -> :wat::test::TestResult)
     (:wat::test::run-hermetic ~body)))
```

Replace the current `run-sandboxed-ast` shape (lines ~310-318). Verify:
- Workspace post-fix: 2243 + N (new probes) / **0 failed** (all 13 V5-blockers pass)
- 4 Gap G probes still pass
- All 11 prior substrate probes still pass

If V5 retry STILL fails — STOP and report. The diagnose may have missed something; surface as honest delta.

### Phase 4 — Verify

```bash
# New Gap J probes
cargo test --release --test probe_register_types_splice_aware

# Phase E V5 retry — the 13 previously-blocked tests
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result"
# Expected: 2243 + N - 0 / 0 failed
```

## Scope (what's IN)

- `register_types` extended to recurse into top-level `do` + `let` body forms
- `register_stdlib_types` similarly extended (mirror)
- 7+ probes in `tests/probe_register_types_splice_aware.rs` (one per type-decl kind in do; let-body equivalent; nested do; end-to-end usage)
- Phase E V5 deftest macro rewrite (apply the V4 BRIEF target shape) — the proof that the fix closes all 3 V5 patterns
- Workspace at 2243 + N / 0 failed

## Scope (what's OUT)

- Any change to `expand_alias` / `reduce` / `unify` — they're correct; the bug was upstream (alias missing from TypeEnv)
- Any change to `preregister_fn_defs_in_do` — the existing splice machinery for fn defs stays
- Any change to Gap F-3's closure type-registry inheritance — it stays; with the parent's TypeEnv now complete, F-3 propagates correctly
- Phase F (`run-sandboxed-*` retirement) — separate slice, gates on V5 retry success
- Slice 4 (destructive reap) — separate slice
- Phase H (clippy/rustc warning sweep) — separate slice
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `register_types` walks top-level do body; type decls nested inside register in TypeEnv | grep + read |
| B | `register_types` walks top-level let body (items[2..]); type decls register | grep + read |
| C | `register_stdlib_types` extended in parallel for substrate-baked stdlib forms | grep + read |
| D | 7+ probes in `tests/probe_register_types_splice_aware.rs` pass | cargo test |
| E | Phase E V5 deftest macro rewrite applied; all 13 previously-failing tests pass | cargo test |
| F | Workspace at 2243 + N / 0 failed; no regression for any existing test | full test run |

**6 rows.** All must PASS.

## Predicted runtime

**60-90 min sonnet.** Substantive substrate work (the splice-recursion helpers + their integration) but mechanical (mirrors existing patterns in `preregister_fn_defs_in_do`). V5 retry is the load-bearing verification.

**Hard cap:** 180 min (2×). ScheduleWakeup at T+10800s.

## Constraints (hard)

- DO NOT modify `expand_alias` / `reduce` / `unify` — substrate machinery is correct; gap is upstream
- DO NOT modify `preregister_fn_defs_in_do` / `_in_let` — existing splice machinery for fn defs stays
- DO NOT modify Gap F-3's `extract_closure` type-registry inheritance
- DO NOT extend to retire `run-sandboxed-*` (Phase F's job)
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit / push / git add / git restore
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- **DO NOT work around substrate defects.** If a test hangs, fails, or behaves unexpectedly and your diagnose points at a substrate-level issue (not just your fix being wrong), STOP and pinpoint. The recovery doc § FM 5 — "volunteering a workaround instead of stopping" — applies aggressively here.
- **USE TIMEOUTS on every cargo test invocation.** Wrap workspace test runs in `timeout 600 cargo test ...` (10 min cap). The previous attempt produced orphaned wat-test processes (PPID 1, hung for >1h) when tests deadlocked without timeout. If a test hangs even with timeout, that's substrate evidence; report it.
- If V5 retry still fails after fix + probes pass, do NOT keep trying. Report Mode B (substrate defects pinpointed). The mission is honest closure, not forced V5 passes.

## Honest delta categories (anticipated)

1. **Nested do recursion termination** — when do contains do, the recursion needs to bound. Trivial via the natural tree-walk; surface the choice.
2. **Form preservation after extraction** — when types are stripped from a do's children, what does the do form look like in `rest`? Probably still a do with the remaining children. Surface the reconstruction shape + edge case "all children were type decls" (resulting do has only the keyword).
3. **Error span preservation** — when nested type-decl errors arise, the span should point to the actual decl, not the enclosing do. Verify via probe.
4. **Substrate stdlib coverage** — `register_stdlib_types` runs on baked stdlib forms before `register_types` runs on user forms. Does the splice-recursion need different rules for stdlib? Probably not; same recursion, different register call.
5. **V5 retry honest deltas** — if V5 still has any failures after the fix, document each one. The hypothesis says all 3 patterns close; surface any that don't (probably indicates pattern was a manifestation of a different gap).

## Cross-references

- `5d82e92` — deftest-hermetic Path E migration (precedent for V5's shape)
- `2e57827` — Gap I-B (last Phase 2a slice)
- `f2de549` — Phase E V4 SCORE (original V4 failure analysis)
- `dc96c7e` — Phase E V4 BRIEF (target shape for the V5 retry)
- Memory `feedback_diagnose_before_spec.md` — the discipline that paid off
- Memory `feedback_no_speculation.md` — the discipline behind the diagnose round

After Gap J + V5 retry ship: Phase 2b's substantive substrate work is done; Phase F (run-sandboxed-* retirement) + Slice 4 (reap) + Phase H (warnings) + Slice 5 (INSCRIPTION) are mechanical closure paperwork.
