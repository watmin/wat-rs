# DESIGN — Stone 241.5 — runtime dispatch wiring; defclause `&` rest-binder integration; 237.8b Gate 1 unblock

**Status:** READY (sub-DESIGN). The Stone 241.4 follow-up that the named-deferral promised. Single substrate site; legacy flat runtime.rs; vigilia gate does NOT apply.

## Why this stone

Stone 241.4 settled the storage foundation:
- Canonical parser parses `& name <- :T` when `allow_rest_binder: true` → `ArgSpec.rest_param: Some((name, ty))`
- A4 (`parse_defclause_args` inlined) sets `allow_rest_binder: true`
- `Clause` struct gained `rest_param: Option<(String, TypeExpr)>` (runtime.rs:704)
- Parser threads it through `parse_defclause_clause` into the Clause's storage

What's MISSING (per Stone 241.4 STOP-6 honest surface): the **runtime dispatch** that CONSUMES `Clause.rest_param`. The dispatcher (`eval_call_to_defclause_with_vals` at runtime.rs:7198) currently:
- Treats arity as STRICT EQUALITY: `declared_arity != called_arity` → fail (line 7216)
- Binds only fixed-position args (line 7262 loop)
- Doesn't extract trailing rest values; doesn't construct a Vector; doesn't bind at rest_param.name

Per Stone 241.4 SCORE STOP-6: ~40-60 lines of mechanical wiring required across arity logic + rest extraction + rest type check + Vector construction + scope binding.

**probe 237.8b Gate 1** (`gate_1_defclause_supports_rest_binder`) currently `#[ignore]`'d with named-Stone-241.5-follow-up. Stone 241.5 un-ignores; Gate 1 passes; arc 237.8b's Gates 2-4 + mint-confirmers open.

## What this stone delivers

ONE substrate site change + new FM 2-bis probe + 237.8b Gate 1 un-ignore.

### S1 — Variadic-min arity check in `eval_call_to_defclause_with_vals`

Replace strict-equality arity check (around runtime.rs:7216) with a variant-aware check:

```rust
let fixed_arity = clause.args.len();
let has_rest = clause.rest_param.is_some();

let arity_ok = if has_rest {
    called_arity >= fixed_arity            // variadic-min: at least fixed args
} else {
    called_arity == fixed_arity            // strict (current behavior preserved)
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

### S2 — Fixed-arg type check stays unchanged

Existing loop (lines 7231-7244) iterates `clause.args.iter().zip(vals.iter())` — already correct for the fixed prefix. `zip` naturally stops at the shorter of the two; when `called_arity > fixed_arity`, the rest values aren't iterated here. **No change to S2's logic** beyond what S1 admits.

### S3 — Rest-binder element type check (per rest value)

After the fixed-arg type-check passes, when `clause.rest_param.is_some()`:

```rust
if let Some((_rest_name, rest_ty)) = &clause.rest_param {
    // Extract the element type T from Vector<T>.
    let elem_ty = match rest_ty {
        TypeExpr::Parametric { head, args }
            if head == "wat::core::Vector" && args.len() == 1
                => &args[0],
        _ => {
            // Stone 241.4 parser accepts any type at rest slot. If a non-Vector
            // type reaches here, surface as substrate failure (the parser should
            // have enforced Vector<T>; if it didn't, that's a follow-up arc bug).
            // For defensive correctness: treat as failure for THIS clause; try next.
            attempted.push(ClauseAttempt {
                clause_index: clause_idx,
                declared_arity: fixed_arity,
                declared_arg_types,
                failure_reason: ClauseFailureReason::ArgTypeMismatch {
                    position: fixed_arity,                       // rest slot position
                    expected: "Vector<T>".to_string(),
                    got: crate::check::format_type(rest_ty),
                },
            });
            continue;
        }
    };

    // Each rest value must match the element type.
    let rest_type_mismatch = vals[fixed_arity..].iter().enumerate()
        .find_map(|(rest_pos, val)| {
            if value_matches_type_by_name(val, elem_ty) {
                None
            } else {
                Some((
                    fixed_arity + rest_pos,
                    crate::check::format_type(elem_ty),
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
}
```

### S4 — Bind rest values as `Value::Vector` in the scope

After the existing fixed-arg binding loop (lines 7262-7269), add:

```rust
if let Some((rest_name, _rest_ty)) = &clause.rest_param {
    let rest_vals: Vec<Value> = vals[fixed_arity..].to_vec();
    let rest_vec = Value::Vector(/* construct HolonVector from rest_vals; sonnet finds the constructor */);
    scope = scope.child().bind(
        rest_name.clone(),
        list_span.clone(),
        TrackedValue::from(rest_vec),
    ).build();
}
```

Sonnet investigates how `Value::Vector` is constructed from `Vec<Value>` (search for existing `Value::Vector(` constructors; likely a `HolonVector::from(...)` or similar). **STOP-6 if construction requires more than ~10 lines** (e.g., type-tag threading; element-conformance re-check; new HolonVector ctor).

### S5 — Un-ignore probe 237.8b Gate 1

Remove the `#[ignore = "..."]` attribute from `gate_1_defclause_supports_rest_binder` (probe_arc237_8b_defclause_arithmetic.rs:85-86). The test should now PASS after S1+S3+S4 ship.

If Gate 1 still fails after the substrate changes, surface as STOP-10 (deeper integration than the BRIEF anticipated; honest delta in SCORE).

### S6 — New FM 2-bis probe (`tests/probe_arc241_stone5_defclause_rest_dispatch.rs`)

Behavioral-parity-style probe (5-7 contracts) verifying dispatch:

| # | Contract | Source |
|---|---|---|
| 1 | Variadic-min: defclause with rest, called with fixed+N values; result computed via rest | `(defclause ([first <- :i64 & rest <- :Vector<:i64>] -> :i64 (...))) call(1,2,3,4)` |
| 2 | Empty rest: called with exactly fixed-arity; rest is empty Vector | call with just the fixed args |
| 3 | Under-supply errors (ArityMismatch) | called with less than fixed-arity |
| 4 | Rest element type mismatch errors | rest contains wrong-type value |
| 5 | Mixed clause set: fixed clause + rest clause; correct dispatch | first match wins per arity |
| 6 | Fixed-only clause still works (regression) | strict equality preserved when rest_param is None |
| 7 | Gate 1 verification (mirror Gate 1's setup; assert result == 10) | redundant with 237.8b Gate 1 but local to this probe |

Sonnet implements; can also reuse Gate 1's `try_compute` helper pattern from 237.8b.

## Locked decisions

### D1 — Variadic-min arity semantics

`called_arity >= fixed_arity` when `clause.rest_param.is_some()`; strict equality otherwise. Under-supply (below fixed_arity) is always ArityMismatch — rest-binder does NOT allow fewer than the fixed args.

### D2 — Element-type check per rest value (not Vector-type check on the whole)

`rest_param.1` is documented as `Vector<T>`; we extract `T` from `TypeExpr::Parametric { head: "wat::core::Vector", args }` and check each `vals[fixed_arity..]` value against `T`. NOT checking whole-Vector type (we're constructing the Vector from values; the check is per-element).

### D3 — Empty rest case: bind empty Vector

When `called_arity == fixed_arity` AND `rest_param.is_some()`: rest_vals is `vec![]`; bind an empty Vector at rest_param.name. The body can call `length` on rest and get 0.

### D4 — Failure reasons reuse existing `ClauseFailureReason` variants

No new variants minted. `ArityMismatch` covers under-supply; `ArgTypeMismatch` covers per-element rest-type failures with `position = fixed_arity + rest_index`. The diagnostic surface tells the user where the problem is.

### D5 — Defensive non-Vector rest type → per-clause failure

If Stone 241.4's parser somehow accepts a non-Vector type at the rest slot (it shouldn't; the syntactic form is `& name <- :T` where T can be anything; type-validity is the check layer's concern, not the parser's), the dispatcher fails this clause with `ArgTypeMismatch` citing `expected: Vector<T>`. NOT a substrate panic; honest per-clause failure.

### D6 — Check layer integration OUT OF SCOPE

The check layer (src/check.rs) might need to bind `rest_param` as `Vector<T>` during clause-body type-check (so body referring to `rest` knows its type). Stone 241.5's scope is RUNTIME dispatch only. **If check layer fails on rest-binder body type-check**: surface as STOP-6 honest delta; defer to Stone 241.6 (or queue as follow-up arc).

Probe 237.8b Gate 1 will reveal whether check-layer integration is needed: if the wat program type-checks at startup successfully (with the runtime substrate changes in place) and computes `10`, integration is sufficient. If type-check fails at startup before runtime dispatch even fires, check-layer work is needed.

### D7 — Vigilia-gate doctrine does NOT apply

`src/runtime.rs` is legacy flat substrate, not a `src/<noun>/` namespaced home. Per `feedback_namespaced_home_vigilia_gate` D9 + `feedback_ward_zone_comms_only`: gate doctrine scoped to namespaced homes. Stone 241.5 commits on SCORE-green.

### D8 — `src/argspec/` UNCHANGED

The canonical home is exceptional + rune-free. Stone 241.5 doesn't touch it. Substrate changes confined to `src/runtime.rs` (dispatch) + tests.

### D9 — No `Clause` struct changes

Stone 241.4 added `Clause.rest_param: Option<(String, TypeExpr)>` (the field name `rest_param` is decided; Stone 241.5 just READS it). No struct field additions in 241.5.

### D10 — Probe 237.8b Gate 1 PASSES post-stone

The integration test. If Gate 1 still RED after S1-S4: STOP-10; surface as honest delta + deeper integration needed.

---

## Trap-door audit

### T1 — Extracting `T` from `Vector<T>` TypeExpr

`TypeExpr::Parametric { head: "wat::core::Vector", args }` is the expected shape. Defensive against non-Vector (D5).

### T2 — `Value::Vector` construction from `Vec<Value>`

Sonnet finds the constructor by grep'ing existing `Value::Vector(` sites. Likely uses `HolonVector::from_vec(...)` or similar. **STOP-6 if requires more than ~10 lines** (e.g., new HolonVector ctor).

### T3 — Body's `rest` reference type-checking

When the clause body references `rest`, it expects `Vector<T>`. Stone 241.4 stored rest_param as `Vector<T>` (the full type), so binding it as a typed value should compose. **If check layer rejects this** (because rest's TypeExpr isn't propagated to the type environment during clause-body check), surface as STOP-6.

### T4 — Mixed clause-set dispatch order

When a ClauseSet has both fixed and rest clauses, the dispatcher tries them in declaration order. First-match-wins. A `[x <- :i64]` clause matches `call(1)` before a `[x <- :i64 & rest <- :Vector<:i64>]` clause does. Order matters; users author clauses with most-specific first. **No change to dispatch semantics**; this is regression behavior.

### T5 — `value_matches_type_by_name` on parametric Vector elements

The element type extracted from `Vector<T>` could itself be parametric (e.g., `Vector<Vector<:i64>>` → element type is `Vector<:i64>`). `value_matches_type_by_name` should handle this — it's the same function fixed-arg type check uses. No new code needed.

### T6 — Rest of arc 237.8b (Gates 2-4 + mint-confirmers)

Stone 241.5 unblocks 237.8b Gate 1 ONLY. Gates 2-4 (defclause arg-type dispatch; 0-ary body literal inference; per-Type ordering primitives) are SEPARATE concerns — arc 237.8b's main strike, not Stone 241.5's. After Stone 241.5, arc 237.8b reopens with Gates 2-4 + mint-confirmers as its own next-strike.

### T7 — `ClauseAttempt` diagnostics for variadic case

When dispatch fails on a variadic clause, the failure message shows `expected: fixed_arity` but the user called with `called_arity` (could be MORE than fixed). The current ArityMismatch shows `expected: fixed_arity` which is technically true (rest takes the rest); user-friendliness could improve (e.g., "expected at least N args"). **DEFER**: keep current shape; mini-arc later if user-feedback warrants.

### T8 — Test cascade depth

Per Stone 241.2/241.3/241.4 calibration: substrate tests assert structurally. Stone 241.5's substrate change is a NEW BEHAVIOR (variadic dispatch); tests that assert "rest-binder rejected" no longer hold IF defclause is set to accept rest-binder. Existing defclause tests use clauses without rest_param; should be UNAFFECTED.

### T9 — Clippy

`vals[fixed_arity..]` slicing may trigger clippy. Defensive: use `vals.iter().skip(fixed_arity)` or `&vals[fixed_arity..]` per idiom.

### T10 — TypeExpr import

Add `use crate::types::TypeExpr;` if not already in `runtime.rs`. Sonnet checks.

---

## STOP triggers (REJECTION)

1. **STOP-1** — Compile errors not traced to the migration sites
2. **STOP-2** — Lib baseline regression below 834
3. **STOP-3** — 60 min elapsed
4. **STOP-4** — `holon-rs` touched
5. **STOP-5** — Files outside `src/runtime.rs`, `tests/probe_arc241_stone5_*`, `tests/probe_arc237_8b_defclause_arithmetic.rs` (un-ignore), SCORE doc, and test files with assertion updates. `src/argspec/*` MUST stay unchanged; `src/lib.rs` MUST stay unchanged; Stone 241.x probes (1/2/3/4) MUST stay at their current PASS counts.
6. **STOP-6** — Scope creep:
   - Check layer changes > ~10 lines (defer to Stone 241.6)
   - New ClauseFailureReason variants (D4 reuses existing)
   - New Clause struct fields (D9 — already added in 241.4)
   - New ArgSpecError variants
   - `Value::Vector` construction requires > ~10 lines (new HolonVector ctor)
7. **STOP-7** — Stone 241.5 probe < N/N PASS
8. **STOP-8** — Stone 241.x (1/2/3/4) probes regress; arc 237/238 probes regress
9. **STOP-9** — Clippy > 904
10. **STOP-10** — Gate 1 STILL RED after un-ignore (deeper integration needed; surface as gap; possibly Stone 241.6)

---

## FM 2-bis evidence

`tests/probe_arc241_stone5_defclause_rest_dispatch.rs` (NEW). Behavioral-parity-with-extension. The probe verifies dispatch behavior that doesn't exist at HEAD (current code rejects all `&` rest-binder dispatch via ArityMismatch since variadic-min isn't implemented).

**Pre-stone**: contracts that test SUCCESS paths (variadic call resolves; rest collected as Vector) will FAIL at HEAD (the substrate either rejects the parse [pre-241.4] or rejects the call [post-241.4 without dispatch]).

**Post-stone**: probe passes N/N + 237.8b Gate 1 passes (the integration test).

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.5.md`. Mirror Stone 241.3's SCORE shape (since gate doctrine doesn't apply):

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~10 rows
- Structural verification ~5 rows (dispatch code present; un-ignore landed; new probe passes; Gate 1 passes; src/argspec/ untouched)
- Code shape: final dispatch body (verbatim)
- Honest deltas (Vector construction approach; check-layer integration status)
- Cascade depth note
- **PHASE 1 TRULY CLOSED**: argspec parser shape complete (241.4) + runtime dispatch wires (241.5) → defclause has full rest-binder support; 237.8b Gate 1 green; arc 237.8b unpauses
- NO Vigilia Convergence section (gate doctrine doesn't apply per D7)

---

## Calibration

**Target band:** 20–40 min Mode A.
**Upper bound:** 60 min (STOP-3).

**Surface estimate:**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/runtime.rs` (eval_call_to_defclause_with_vals dispatch) | (current) | (+~45 lines) | **+45** |
| `tests/probe_arc241_stone5_defclause_rest_dispatch.rs` (NEW) | 0 | ~150 | **+150** |
| `tests/probe_arc237_8b_defclause_arithmetic.rs` | (with #[ignore]) | (no #[ignore]) | **-1** |
| **Net delta** | — | — | **~+194 lines** |

**Confidence: HIGH.** Stone 241.4 settled the storage foundation; Stone 241.5 is the mechanical "consume what's stored" stone. The semantic decisions (variadic-min arity; rest as Vector; first-match-wins; per-element check) are uncontroversial. Risk concentrated in T2 (Value::Vector construction depth) and T3 (check-layer integration). Both have STOP-6 escape hatches.

**Per `feedback_stone_briefs_cite_prior_score`**: BRIEF cites Stone 241.4 SCORE for storage foundation; cites probe 237.8b Gate 1 for the integration test.

---

## What this unblocks

**Arc 237.8b** unpauses fully: Gate 1 green; Gates 2-4 + mint-confirmers proceed. Arc 237.8b was the ORIGINAL blocker that drove arc 241's opening (six stones + 241.5 = seven stones ago).

**Phase 1 of arc 241 TRULY CLOSED** after this: canonical parser shape complete + runtime dispatch wired + defclause has full rest-binder semantics + Gate 1 integration confirmed.

**Phase 2 of arc 241** opens: 241.6 mints `:wat::runtime::metadata-of` reflection verb; 241.7 optional `{...}` metadata-map on `def`/defn.

---

## Cross-references

- `SCORE-STONE-241.4.md` § Vigilia Convergence — the storage foundation; Clause.rest_param + Clause.fixed_params now in place; Stone 241.5 consumes
- Probe `tests/probe_arc237_8b_defclause_arithmetic.rs:86` — Gate 1 with named-Stone-241.5-follow-up ignore reason; un-ignore is the integration verification
- `AUDIT.md` § A4 + ParseOptions — defclause's allow_rest_binder: true at parse_defclause_clause now flows into runtime dispatch
- `feedback_namespaced_home_vigilia_gate` — gate does NOT apply (legacy flat runtime.rs)
- `feedback_no_pre_existing_excuse` + FM 11 — Gate 1 deferral being CLOSED by this stone (named-follow-up fulfilled)
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 241.4 SCORE; structural foundation
- `feedback_sonnet_writes_substrate` — orchestrator briefs + scores; sonnet writes the dispatch logic
