# BRIEF — Stone 241.5 — runtime dispatch wiring; defclause `&` rest-binder integration

You are sonnet (the Shadowdancer). Stone 241.4's named-follow-up — runtime dispatch in `eval_call_to_defclause_with_vals` consumes the `Clause.rest_param` that Stone 241.4 stored. Gate 1 flips green; arc 237.8b unpauses.

## What to do

Single substrate file change in `src/runtime.rs` (~45 lines net) + new probe + 237.8b Gate 1 un-ignore.

### S1+S3+S4 — Wire dispatch in `eval_call_to_defclause_with_vals` at runtime.rs:7198

The dispatcher does (currently):
1. Arity check (strict equality) — line 7216
2. Type check on each fixed arg — line 7231
3. Bind args to scope — line 7262
4. Guard, body, ensure (preserved unchanged) — lines 7271+

**Your changes** insert rest-binder handling at steps 1, 2/3 (new), and 3:

#### Step 1 (S1) — Variadic-min arity

Replace the strict-equality check:

```rust
let fixed_arity = clause.args.len();
let has_rest = clause.rest_param.is_some();

let arity_ok = if has_rest {
    called_arity >= fixed_arity
} else {
    called_arity == fixed_arity
};

if !arity_ok {
    attempted.push(ClauseAttempt {
        clause_index: clause_idx,
        declared_arity: fixed_arity,
        declared_arg_types,
        failure_reason: ClauseFailureReason::ArityMismatch {
            expected: fixed_arity,
            got: called_arity,
        },
    });
    continue;
}
```

#### Step 2 (existing — unchanged)

The fixed-arg type check at lines 7231-7244 stays. `zip` naturally stops at fixed_arity when called_arity is larger.

#### Step 2.5 (S3 — NEW) — Rest-binder element type check

Inserted AFTER the fixed-arg type check passes, BEFORE the bind step:

```rust
let rest_elem_ty_opt: Option<&TypeExpr> = if let Some((_rest_name, rest_ty)) = &clause.rest_param {
    // Extract element type T from Vector<T>.
    let elem = match rest_ty {
        crate::types::TypeExpr::Parametric { head, args }
            if head == "wat::core::Vector" && args.len() == 1
                => &args[0],
        _ => {
            // Defensive: parser should have enforced Vector<T>; if not, fail this clause.
            attempted.push(ClauseAttempt {
                clause_index: clause_idx,
                declared_arity: fixed_arity,
                declared_arg_types,
                failure_reason: ClauseFailureReason::ArgTypeMismatch {
                    position: fixed_arity,
                    expected: "Vector<T>".to_string(),
                    got: crate::check::format_type(rest_ty),
                },
            });
            continue;
        }
    };

    // Per-element type check.
    let rest_type_mismatch = vals[fixed_arity..].iter().enumerate()
        .find_map(|(rest_pos, val)| {
            if value_matches_type_by_name(val, elem) {
                None
            } else {
                Some((
                    fixed_arity + rest_pos,
                    crate::check::format_type(elem),
                    val_type_path(val).to_string(),
                ))
            }
        });

    if let Some((pos, expected, got)) = rest_type_mismatch {
        attempted.push(ClauseAttempt {
            clause_index: clause_idx,
            declared_arity: fixed_arity,
            declared_arg_types,
            failure_reason: ClauseFailureReason::ArgTypeMismatch {
                position: pos,
                expected,
                got,
            },
        });
        continue;
    }
    Some(elem)
} else {
    None
};
```

#### Step 3 (S4) — Bind rest as `Value::Vector` in scope

After the existing fixed-arg binding loop (lines 7262-7269), add:

```rust
if let Some((rest_name, _rest_ty)) = &clause.rest_param {
    let rest_vals: Vec<Value> = vals[fixed_arity..].to_vec();
    let rest_vec_value: Value = /* construct from rest_vals — see Vector construction note below */;
    scope = scope.child().bind(
        rest_name.clone(),
        list_span.clone(),
        TrackedValue::from(rest_vec_value),
    ).build();
}
```

**Vector construction**: grep `Value::Vector(` in runtime.rs to find the existing constructor pattern. Likely `Value::Vector(HolonVector::from(rest_vals))` or similar — sonnet investigates. **STOP-6 if construction requires more than ~10 lines** (e.g., new HolonVector ctor; type-tag threading; element re-conformance).

### S5 — Un-ignore probe 237.8b Gate 1

In `tests/probe_arc237_8b_defclause_arithmetic.rs:85-86`, REMOVE the entire `#[ignore = "Stone 241.4 shipped..."]` attribute line. Final shape:

```rust
#[test]
fn gate_1_defclause_supports_rest_binder() {
    // body unchanged
}
```

The test should now PASS green. If it fails: STOP-10 honest delta (deeper integration needed; possibly Stone 241.6).

## Discipline

- **`src/argspec/*` UNCHANGED.** Canonical home is exceptional + rune-free post-Stone-241.4.
- **`src/lib.rs` UNCHANGED.**
- **`src/check.rs`** — likely UNCHANGED but may need defclause body type-check integration (rest as Vector<T>); STOP-6 if changes > ~10 lines.
- **Stone 241.1/241.2/241.3/241.4 probes UNCHANGED** at their current PASS counts.
- **Clause struct UNCHANGED.** Stone 241.4 added `rest_param`; Stone 241.5 reads it; no new fields.
- **No new ClauseFailureReason variants.** Reuse ArityMismatch + ArgTypeMismatch.
- **No new ArgSpecError variants** (Stone 241.4 already shipped the 7 variants).
- **`use crate::types::TypeExpr;`** may need to be added at runtime.rs top — sonnet checks.
- **No `cargo run`; no wrapper scripts; just `cargo test/build/clippy`.**

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.5.md` — this doc
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.5.md` — D1-D10 + T1-T10 + STOP
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.4.md` § Vigilia Convergence — storage foundation
5. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 699-733 (Clause + ClauseSet struct definitions; rest_param field on Clause)
6. `/home/watmin/work/holon/wat-rs/src/runtime.rs` lines 7178-7350 (eval_call_to_defclause + eval_call_to_defclause_with_vals — your strike target)
7. `/home/watmin/work/holon/wat-rs/src/types.rs` lines 65-100 (TypeExpr enum — Parametric variant)
8. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone5_defclause_rest_dispatch.rs` — 8-contract FM 2-bis probe (3 PASS / 5 FAIL at HEAD; post-stone 8/8)
9. `/home/watmin/work/holon/wat-rs/tests/probe_arc237_8b_defclause_arithmetic.rs` lines 60-110 (Gate 1 source; un-ignore target)
10. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/EXPECTATIONS-STONE-241.5.md` — scorecard

## Implementation sketch

1. Read substrate + probes
2. Baseline check:
   - `cargo test --release --lib -p wat` (expect 834 PASS)
   - `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch` (expect 3 PASS / 5 FAIL at HEAD)
   - `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1` (expect ignored)
3. Find `Value::Vector` constructor: `grep "Value::Vector(" src/runtime.rs | head` — find an existing site that builds Vector from Vec<Value>
4. **S1**: replace strict-equality arity check with variadic-min
5. **S3**: insert rest-binder element type check
6. **S4**: insert rest-binder scope bind (Vector construction)
7. Run Stone 241.5 probe; iterate until 8/8 PASS
8. **S5**: remove `#[ignore]` from Gate 1; verify Gate 1 passes
9. Run lib tests; identify any cascade; update assertions as honest deltas
10. Final verification:
    - `cargo test --release --lib -p wat` (≥834 PASS)
    - `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch` (8/8)
    - `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1` (1 PASS)
    - All Stone 241.x probes preserved (1: 15/15; 2: 10/10; 3: 6/6)
    - Arc 237/238 probes preserved
    - `cargo build --release --tests --workspace` clean
    - `cargo clippy --release` ≤ 904
11. Write `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.5.md`
12. **DO NOT COMMIT.** Orchestrator commits.

## STOP triggers — REJECTION

1. Compile errors not traced to migration sites
2. Lib < 834
3. 60 min elapsed
4. holon-rs touched
5. Files outside `src/runtime.rs`, `tests/probe_arc241_stone5_*`, `tests/probe_arc237_8b_defclause_arithmetic.rs`, SCORE doc, and test files with assertion updates touched. `src/argspec/*` + `src/lib.rs` MUST stay unchanged.
6. Scope creep: check-layer changes > ~10 lines (STOP-6; defer to Stone 241.6); Value::Vector construction > ~10 lines; new ClauseFailureReason variants; new Clause fields; new ArgSpecError variants
7. Stone 241.5 probe < 8/8 PASS
8. Stone 241.x probes regress; arc 237/238 probes regress
9. Clippy > 904
10. Gate 1 STILL RED after un-ignore (honest delta; surface as gap)

## SCORE doc spec — `SCORE-STONE-241.5.md`

Mirror SCORE-STONE-241.3.md structural shape (no vigilia section; gate doctrine doesn't apply):

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification ~5 rows (variadic-min arity present; rest type check present; rest bind present; Gate 1 #[ignore] removed; src/argspec/ untouched)
- Migration audit (runtime.rs dispatch delta)
- Final post-stone dispatch body (verbatim)
- Honest deltas (Vector construction approach; check-layer integration status)
- Cascade depth note
- **PHASE 1 TRULY CLOSED inscription**: argspec parser shape complete (241.4) + runtime dispatch wires (241.5) → defclause has full rest-binder; 237.8b Gate 1 green; arc 237.8b unpauses
- NO Vigilia Convergence section (per DESIGN D7)

## Post-strike

When SCORE-STONE-241.5.md is written and verification passes, return with a one-paragraph status covering: dispatch wired; Stone 241.5 probe 8/8; Gate 1 status (green or surfaced gap); Vector construction approach taken; check-layer integration status.

Stone 241.4's named-follow-up. The wait is over. Strike clean.
