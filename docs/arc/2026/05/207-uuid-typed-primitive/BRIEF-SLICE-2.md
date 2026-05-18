# BRIEF — Arc 207 Slice 2: mint `:wat::core::Uuid` + 6 verbs + edn_shim fix

**Predecessors:** Slice 1 SCORE (audit + shape decision) at commit `<this commit>`. DESIGN updated forward to reflect slice 1's findings (slice 3 edn_shim fix folded into this slice; `values_equal` arm explicitly required; canonical-only parse strictness; no hashmap_key in slice 2).

**This slice is the substantive substrate work for arc 207.** It mints the `:wat::core::Uuid` type at the wat substrate level (option (c) — new `Value::wat__core__Uuid(uuid::Uuid)` variant per Pattern B precedent), adds six verbs, fixes the EDN shim read+write arms, and lands one new test file. It does NOT retire arc 206's namespace verbs (that's slice 3); it does NOT ripple consumers (that's slice 4).

## Source of truth: SCORE-SLICE-1 § Slice 2 substrate surface checklist

Read `docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-1.md` § "Slice 2 substrate surface checklist" — that section is the canonical 20-item checklist for this slice. Every item there is in scope. This BRIEF references the checklist by item number rather than re-listing.

**Slice 2 = items 1–20 from SCORE-SLICE-1 checklist, plus item 7 (edn_shim WRITE arm — folded in here per honest delta 1).**

Specifically: items 1–6 (runtime.rs variant + arms; edn_shim read), 7 (edn_shim write — was in honest delta), 8–12 (check.rs schemes), 13–17 (eval handlers), 18 (dispatch wiring), 19 (no types.rs change), 20 (new test file).

## Confirming decisions surfaced by slice 1's honest deltas

- **Delta 1: edn_shim fix folds into slice 2.** Confirmed. Items 6 (read) + 7 (write) ship here. Original DESIGN slice 3 → folded.
- **Delta 2: `Uuid/from-string` canonical-only.** Confirmed. Return `None` for uppercase, urn:uuid: prefix, braced, or otherwise non-canonical. If `uuid::Uuid::parse_str` succeeds, validate canonical form (lowercase, exactly 8-4-4-4-12 hyphenated) before returning `Some`. Sonnet implements; test coverage explicit per item 20.
- **Delta 3: no `hashmap_key` arm.** Confirmed. Out of slice 2 scope. If slice 4's consumer ripple surfaces a real need, add it then; don't pre-add.
- **Delta 4: `values_equal` arm explicit.** Confirmed. Item 3 in the checklist.
- **Delta 5: `Uuid/from-string` returns `Option`.** No action needed; already correct.

## Verification gate (sonnet's first action)

Before any code edits, sonnet:

1. **Baseline check.** `git status --short` should be clean (only `.claude/worktrees/` harness state). `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` records the baseline. Expected: 3-4 pre-existing failures (`lifeline_pipe_zero_orphans_across_100_trials`, `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`).
2. **Read SCORE-SLICE-1.md fully.** The audit findings + four-questions justification + 20-item checklist ARE the implementation guide. Don't second-guess shape decision (option (c) is settled).
3. **Confirm the 5 file:line refs in the checklist exist** before relying on them: `runtime.rs:371` (Value enum), `runtime.rs:611` (Duration variant), `runtime.rs:704` (type_name), `runtime.rs:6768` (values_equal), `edn_shim.rs:404` (Edn::Uuid arm). If any drifted, surface; orchestrator decides.

## Implementation order (orchestrator-suggested; sonnet's call to adjust)

Per `feedback_iterative_complexity` build small; per `feedback_test_first` write the test before the impl when possible. Suggested order:

1. **Add the test file first** (item 20) — empty test bodies asserting the surface that will exist. Tests fail compilation (verb names don't resolve) — that's the substrate-as-teacher diagnostic for items 8–18.
2. **runtime.rs items 1–3** (Value variant + type_name + values_equal). Workspace compiles; tests still fail because verbs don't exist.
3. **check.rs items 8–12** (5 scheme registrations). Type checking gates appear in test failures.
4. **string_ops.rs (or new uuid_ops.rs) items 13–17** (5 eval handlers).
5. **runtime.rs item 18** (dispatch wiring). Tests start passing.
6. **edn_shim.rs items 6–7** (read + write arms). Roundtrip test passes.
7. Run full test suite. Confirm workspace baseline preserved.

If sonnet finds a better order (e.g., compile cycle blocks elsewhere), trust sonnet's judgment.

## EDN roundtrip semantics (item 7 design detail)

`value_to_edn_with` arm for `Value::wat__core__Uuid(u)`: produces `OwnedValue::Uuid(*u)` matching the existing `Value::Instant → OwnedValue::Inst` pattern (cited in SCORE audit 2). This means `(:wat::edn::write some-uuid-value)` produces the canonical `#uuid "..."` reader literal. Test coverage: round-trip a value through write+read, assert equality.

## `Uuid/from-string` canonical-strict implementation hint

Sonnet's call on exact mechanism, but a reasonable pattern:

```rust
fn is_canonical_uuid_string(s: &str) -> bool {
    // 36 chars, lowercase hex, hyphens at 8/13/18/23
    s.len() == 36
        && s.as_bytes()[8] == b'-'
        && s.as_bytes()[13] == b'-'
        && s.as_bytes()[18] == b'-'
        && s.as_bytes()[23] == b'-'
        && s.chars().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) { c == '-' }
            else { c.is_ascii_hexdigit() && (!c.is_alphabetic() || c.is_ascii_lowercase()) }
        })
}
```

Then `eval_uuid_from_string`: if `is_canonical_uuid_string(&s) && uuid::Uuid::parse_str(&s).is_ok()`, return `Some`; else `None`. (parse_str is the actual validator; the canonical check rejects accepted-but-non-canonical forms.)

## HARD constraints

- DO NOT touch `crates/wat-edn/` (substrate-of-substrate; arc 207 does not edit it). The wat-edn `Value::Uuid` variant + `#uuid` reader/writer ALREADY EXIST and slice 2 just lights up the path that was waiting.
- DO NOT retire `:wat::core::uuid::*` namespace verbs (that's slice 3). They keep working alongside the new `Uuid/*` verbs through this slice.
- DO NOT touch telemetry's `wat/telemetry/uuid.wat` alias (slice 3 retargets it).
- DO NOT touch arc 203 demos (`wat-tests/counter-service-*.wat`) — slice 4 consumer ripple covers them.
- DO NOT commit. Orchestrator commits atomically after independent verification.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/` (illegal per FM 7-bis).
- DO NOT use `--no-verify` / `--no-gpg-sign`.

## STOP triggers (surface immediately)

1. **`Value::wat__core__Uuid` Rust type-name collision** with anything else in `Value` enum. Surface; orchestrator decides.
2. **Workspace baseline regresses** beyond 4 pre-existing failures. Surface the new failure with diagnostic + file:line; do NOT silently bypass.
3. **`values_equal` arm causes a compile error** in unrelated code (e.g., if there's a `_ =>` exhaustiveness check somewhere that needs adjustment). Surface.
4. **`edn_shim.rs:404` has drifted** since slice 1 audit (different arm, different error, different line). Re-audit and surface what's actually there.
5. **`is_canonical_uuid_string` rejects valid `uuid::Uuid::parse_str` outputs.** That would be a bug in the canonical check; surface + adjust.
6. **Test coverage for items 20 (8 cases) can't all pass** — surface which case fails and why; orchestrator decides whether to adjust the test or the impl.

## SCORE methodology

`docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-2.md` with these rows (atomic YES/NO; no "medium"):

| Row | Evidence |
|---|---|
| A — Verification gate passed (baseline clean, 5 file:line refs confirmed) | Each check's command + result inscribed |
| B — All 20 SCORE-SLICE-1 checklist items completed + item 7 (edn write) added | Each item with file:line of the new code |
| C — Workspace baseline preserved at ≤4 pre-existing failures | `cargo test --release --workspace --no-fail-fast` output |
| D — All 8 test cases in `tests/wat_arc207_uuid_typed.rs` pass | Test output cited |
| E — `Uuid/from-string` strictness verified: canonical → `Some`, non-canonical (uppercase, urn prefix, braced, garbage) → `None` | Test cases listed |
| F — EDN roundtrip works: `(:wat::edn::write uuid)` produces `#uuid "..."`; `(:wat::edn::read "#uuid \"...\"")` produces typed Uuid value | Roundtrip test passes |
| G — `:wat::core::uuid::*` namespace verbs STILL WORK alongside new `Uuid/*` verbs (arc 206 backward compat through slice 2; slice 3 retires) | Arc 206 tests still pass |
| H — Clippy clean on touched files | `cargo clippy --release -p wat 2>&1 | grep -E "warning|error"` on touched files only |

## Honest delta watch

Surface honestly if:
- Slice 1's checklist had any errors (file:line refs that drifted, items that don't compile cleanly)
- `Uuid/from-string`'s canonical-strict semantics conflict with any existing test's assumptions
- The `values_equal` arm's placement requires reordering surrounding arms
- The edn_shim `value_to_edn_with` arm placement requires a specific position relative to other arms (e.g., must come after Instant, or before Map, etc.)
- Any other gap between SCORE-SLICE-1's prediction and the implementation reality

## Time-box

Predicted 60-90 min sonnet. Hard stop 120 min. Larger slice — 20 items + tests + verification — but uniform composition per `feedback_simple_is_uniform_composition`.

## On completion

Return summary: rows passed/failed, total items completed, file:line for each new code addition (especially the Value variant + eval handlers + edn_shim arms), any honest deltas surfaced.

You are launching now. T-minus 0.
