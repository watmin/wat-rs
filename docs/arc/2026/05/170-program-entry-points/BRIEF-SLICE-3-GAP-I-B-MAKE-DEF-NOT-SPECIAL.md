# Arc 170 slice 3 Gap I-B BRIEF — make `def` not special (retire validator def-arm + tighten runtime arm)

**Sonnet.** Substrate fix that closes Gap I-A's honest delta. Gap I-A's predicate `is_declaration_form` correctly includes `def`, but `def` at fn-body do-prefix is blocked at PARENT check-time by `validate_def_position_with_wrapper` (which only catches def specifically — the other 7 declaration forms fall through silently). Gap I-A's probe 1 had to be a unit test of the predicate instead of an end-to-end spawn test; the mixed probe covered 7 of 8 forms.

Gap I-B closes this by **making def behave like its 7 siblings**:
- Retire the validator's `:wat::core::def` arm — def falls through `_ =>` like the other 7
- Tighten def's runtime dispatch arm to emit a position-class error matching `define`'s behavior

Closes a latent arc-157 defect surfaced by Gap I-A: arc 157's runtime arm at `src/runtime.rs:3522` was permissive (evaluates RHS, returns Unit, doesn't register) because the comment explicitly assumed *"this arm is only reached for legal top-level defs"* — the validator was the load-bearing entry guard. With Gap I-A's lift mechanism, that assumption broke. Runtime arm must become self-sufficient.

User direction 2026-05-13 (after probing why def was special):
> *"making it not special feels best."*

## Backstory in one sentence

Gap I-A's lift covered 5 forms end-to-end but `def` was blocked at parent check-time; Gap I-B closes the asymmetry through retirement + runtime-arm tightening.

## Goal — def behaves like the other 7 declarations end-to-end

**Check-time:** Today validator emits `DefNotTopLevel` for def at NonTopLevel; nothing for the other 7. After: validator silent for all 8 declarations; runtime/freeze-time mechanisms are authoritative.

**Runtime:** Today def at expression position silently returns Unit; define emits `DefineInExpressionPosition`; struct/enum/etc. blocked via `refuse_mutation_forms`. After: def emits a position-class error mirroring define.

**End-to-end:** Today def at fn-body do-prefix in a closure flowing to spawn-process gets check-time `DefNotTopLevel`; user can't compile. After: validator silent, extract_closure lifts def into the prologue (via Gap I-A's predicate), child's startup registers def at top-level, end-to-end spawn works.

## Why this scope (and what's OUT)

This Gap is the SYMMETRY restoration. Three pieces:
1. Retire validator's def arm (deletion)
2. Tighten runtime arm so def at expression position emits an error (replacement)
3. Sweep existing tests that depend on the old behavior

What's OUT:
- Loads (`load-file!` family) and config setters (`config::set-*`) — they're not declarations; their position discipline is separate; out of scope per Gap I-A's architectural intent
- ANY change to `is_declaration_form` itself — Gap I-A's predicate ships unchanged
- ANY change to the lift mechanism in extract_closure — Gap H + Gap I-A's behavior preserved
- deftest-hermetic Path E macro shape — separate slice
- Phase E V5 / Phase F / Slice 4/5 work — Phase 2b territory

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-I-A-IS-DECLARATION-FORM.md`** (commit `8c13631`) — Gap I-A's honest delta names the dependency Gap I-B closes
2. **`docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`** § "Gap I-B and the three ways def was special" — architectural reasoning + four-questions verdict
3. **`src/check.rs`** lines 7039-7164 — `validate_def_position_with_wrapper`; lines 7094-7110 are the `:wat::core::def` arm to retire
4. **`src/runtime.rs`** lines 3520-3540 — the def + define dispatch arms; def's permissive behavior + define's loud error are adjacent for comparison
5. **`src/runtime.rs`** lines 1065 + 1319 — `DefineInExpressionPosition` variant definition + Display impl (the variant to generalize OR retire)
6. **`docs/arc/2026/05/057-substrate-def-form/` (if exists)** — arc 157's original design context for def + position rule

## Implementation path

### Phase 1 — Audit the existing surface (10-15 min)

```bash
# Find every reference to DefNotTopLevel
grep -rn "DefNotTopLevel" src/ tests/ wat-tests/ 2>/dev/null

# Find every reference to DefineInExpressionPosition  
grep -rn "DefineInExpressionPosition" src/ tests/ wat-tests/ 2>/dev/null

# Find tests that exercise def at expression position
grep -rn "def.*not.*top.*level\|def.*expression.*position" tests/ wat-tests/ 2>/dev/null

# Find the CheckError variant definition
grep -n "DefNotTopLevel" src/check.rs
```

Tally:
- How many test sites assert on `DefNotTopLevel`?
- How many sites construct or match `DefineInExpressionPosition`?
- Is `DefineInExpressionPosition` exported / part of the public API surface?

Document the inventory before changing code.

### Phase 2 — Choose the runtime error variant name (5 min)

Sub-decision: should the position error for def-at-expression be:
- **(α)** Mint NEW variant `DeclarationInExpressionPosition(String, Span)` carrying the head; route both `:wat::core::def` and `:wat::core::define` through it; retire `DefineInExpressionPosition` via sweep. Cleanest naming; matches the predicate name. Touches more sites.
- **(β)** Add new variant `DefInExpressionPosition(Span)` just for def. Keep `DefineInExpressionPosition` for define. Smaller change; two near-identical variants left in the substrate.
- **(γ)** Reuse `DefineInExpressionPosition` for def too; update Display to render the actual head. Smallest change; variant name LIES for def.

**Recommendation: (α)** — symmetric with `is_declaration_form`'s naming; eliminates the asymmetry; the rename sweep is small (~1 variant + Display + ~handful of test assertions).

Decision rationale goes in SCORE.

### Phase 3 — Retire validator's def arm (5 min)

In `src/check.rs::validate_def_position_with_wrapper` (line 7094-7110), DELETE the `:wat::core::def` arm. Def falls through to the `_ =>` arm with the other 7 declarations.

### Phase 4 — Tighten runtime arm (10-15 min)

In `src/runtime.rs::eval` dispatch (line 3520-3540), the `:wat::core::def` arm currently:
```rust
":wat::core::def" => {
    // arity check
    let _value = crate::runtime::eval(&args[1], env, sym)?;
    Ok(Value::Unit)
}
```

Replace with position-error emission:
```rust
":wat::core::def" => Err(RuntimeError::DeclarationInExpressionPosition(
    ":wat::core::def".into(),
    list_span.clone(),
)),
```

(Assuming Phase 2 chose option α; adapt for β/γ accordingly.)

### Phase 5 — Mint/rename the error variant (10-15 min)

Per Phase 2's choice:
- **(α)** Mint `DeclarationInExpressionPosition(String, Span)` in `RuntimeError` enum. Update Display to render `"Declaration form `{head}` in expression position"`. Update `:wat::core::define` arm at runtime.rs:3539 to use the new variant carrying `":wat::core::define"`. Retire the old `DefineInExpressionPosition` variant (delete).
- **(β)** Add `DefInExpressionPosition(Span)` alongside the existing variant. Display: `"def form in expression position"`.
- **(γ)** Generalize `DefineInExpressionPosition`'s Display to render whichever form's at fault.

### Phase 6 — Sweep test assertions (15-30 min)

Any test that:
- Asserts `DefNotTopLevel` for def-at-expression-position needs updating — that error no longer fires from check-time. EITHER:
  - The test was checking check-time behavior of def specifically → update to expect the runtime error (or update the test to construct a scenario that still reaches the validator's `_ =>` recursion for the def-deep-in-if case)
  - The test was checking the position discipline → assert on the new runtime error variant
- Constructs `DefineInExpressionPosition` (if Phase 2 chose α) → update to `DeclarationInExpressionPosition`

This sweep may surface tests that need rethinking (their original intent was "def at expression should be caught early"; the new intent is "def at expression position is caught LATE — at runtime if called, or never if dead-code/lifted").

### Phase 7 — New probes (15-20 min)

Create `tests/probe_def_not_special.rs`:

```rust
#[test]
fn probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end() {
    // The probe Gap I-A's probe 1 couldn't deliver.
    // (fn [..] -> :nil (do (:def :x 42) (:x)))
    // Spawned via spawn-process; child registers :x; child returns expected value.
}

#[test]
fn probe_def_at_expression_position_emits_position_error_at_runtime() {
    // def buried inside an if branch in a normally-called fn.
    // Runtime emits DeclarationInExpressionPosition (or chosen variant).
}

#[test]
fn probe_def_at_top_level_still_works() {
    // Regression — def at top-level legal as before.
}

#[test]
fn probe_define_at_expression_position_still_emits_error() {
    // Regression — define's position discipline unchanged; new variant (if α) carries ":wat::core::define".
}

#[test]
fn probe_mixed_declaration_prelude_now_includes_def() {
    // The mixed-prelude probe from Gap I-A — extended to include def in the prefix.
    // Verifies all 8 declaration forms lift together.
}
```

### Phase 8 — Verify

```bash
# New Gap I-B probes
cargo test --release --test probe_def_not_special

# All Gap I-A probes still pass
cargo test --release --test probe_declaration_form_lift

# All Gap H probes still pass
cargo test --release --test probe_closure_body_prelude_lift

# All 11 prior substrate probes
cargo test --release \
  --test probe_do_splice_def --test probe_let_splice_def \
  --test probe_do_splice_define --test probe_let_splice_define \
  --test probe_do_splice_struct --test probe_do_splice_enum \
  --test probe_let_splice_struct --test probe_let_splice_enum \
  --test probe_spawn_process_parent_type \
  --test probe_resolver_quote_awareness \
  --test probe_deftest_hermetic_isolation

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result"
# Expected: 2238 + N - M / 0 failed (N = new probes ≥ 5; M = old DefNotTopLevel-asserting tests updated to new shape — may be 0 if assertions still pass against the new equivalent)
```

## Scope (what's IN)

- Retire `:wat::core::def` arm in `validate_def_position_with_wrapper` (`src/check.rs:7094-7110`)
- Tighten `:wat::core::def` arm in eval dispatch (`src/runtime.rs:3520-3540`) to emit a position error
- Mint/rename error variant per Phase 2 choice (α recommended)
- Sweep existing tests asserting on retired/changed behavior
- 5+ new probes in `tests/probe_def_not_special.rs`
- Workspace at 0 failed

## Scope (what's OUT)

- ANY change to `is_declaration_form` itself — Gap I-A's predicate ships unchanged
- ANY change to extract_closure lift mechanism — Gap H + Gap I-A's behavior preserved
- ANY change to the other 7 declarations' runtime arms (define stays as-is unless Phase 2 chose α; struct/enum/etc. stay unchanged)
- Loads + config setters position discipline — separate concern, out of scope
- deftest-hermetic Path E macro shape rewrite — separate slice
- Phase 2b work (Phase E V5 / Phase F / Slice 4/5) — territory after Phase 2a closes
- `CheckError::DefNotTopLevel` variant itself — if no live emission sites remain after retirement, the variant can be removed in a follow-up sweep (NOT in this slice; that's a separate retirement)
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::core::def` arm in `validate_def_position_with_wrapper` deleted; def falls through `_ =>` like the 7 siblings | grep + read |
| B | `:wat::core::def` arm in eval dispatch emits position-class error (chosen variant per Phase 2); does NOT silently return Unit | grep + read |
| C | Error variant minted/renamed per Phase 2 decision; Display renders accurately for whichever form is at fault; rationale in SCORE | grep + read |
| D | 5+ new probes in `tests/probe_def_not_special.rs` pass: end-to-end spawn lift; runtime position error; top-level regression; define regression; mixed 8-form prelude | cargo test |
| E | All 6 Gap I-A probes + all 5 Gap H probes + all 11 prior substrate probes still pass | cargo test |
| F | `cargo check --release` green; workspace at 2238 + N (- M_updated) / 0 failed | full test run |

**6 rows.** All must PASS.

## Predicted runtime

**60-90 min sonnet.** Mechanical retirement + runtime tightening + variant naming + test sweep. Sweep size unknown until Phase 1's audit; could expand if many tests assert on `DefNotTopLevel` or `DefineInExpressionPosition`.

**Hard cap:** 180 min (2×). ScheduleWakeup at T+10800s.

## Constraints (hard)

- DO NOT modify `is_declaration_form` itself — Gap I-A predicate ships unchanged
- DO NOT modify extract_closure's lift mechanism — Gap H + Gap I-A behavior must be preserved
- DO NOT modify the other 7 declarations' position discipline (the `_ =>` arm in the validator catches them correctly; their runtime arms stay)
- DO NOT add loads / config setters to the changes — separate semantic category, out of scope
- DO NOT touch `docs/arc/` (FM 11 — orchestrator owns paperwork)
- DO NOT commit / push / git add / git restore — orchestrator atomic-commits after scoring
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO NOT silently change the public API surface — if `DefineInExpressionPosition` is exported (Phase 1 audit will reveal), the rename to `DeclarationInExpressionPosition` is API-breaking; document in SCORE
- If a Gap H or Gap I-A regression surfaces (any of the 11 prior probes fails), STOP and report — Gap I-B must be transparent for Gap H/I-A's working forms

## Honest delta categories (anticipated)

1. **Variant naming choice (α/β/γ)** — Phase 2 sub-decision. Recommendation α; surface choice + rationale in SCORE.
2. **`CheckError::DefNotTopLevel` orphan variant** — after retirement, the variant may have no emitters. Should this slice delete the variant, or leave it for a follow-up cleanup? Recommendation: leave; surface as known follow-up if no emitters remain (with affirmative scope-bounding, not deferral language).
3. **Test sweep size** — Phase 1 audit reveals the surface. If LARGE (>10 sites), surface as a separate concern; if SMALL (<10 sites), sweep inline.
4. **Public API impact** — does retiring `DefineInExpressionPosition` break consumers? Surface in SCORE.
5. **Deep-recursion def-violations** — if def appears deep inside an if branch (currently caught by validator's recursion into the if's branches), after retirement those defs don't get check-time caught. They get caught at runtime when the fn is called. This is the symmetric behavior with the other 7; surface as expected.
6. **Anything unexpected** — particularly around tests that may have been structured to depend on the old check-time-fail vs runtime-fail distinction

## Cross-references

- `8c13631` Gap I-A (the slice that surfaced Gap I-B's necessity via honest delta 1)
- `36030c3` Gap H (the lift mechanism Gap I-B's def-fix unblocks)
- arc 157 (the arc that minted def's check-time validator AND the permissive runtime arm assumption Gap I-B is now correcting)
- After Gap I-B ships: Phase 2a complete (F-1, F-3, F-2, G, H, I-A, I-B all ✅); deftest-hermetic Path E macro shape becomes a small wat/test.wat edit; Phase 2b resumes
