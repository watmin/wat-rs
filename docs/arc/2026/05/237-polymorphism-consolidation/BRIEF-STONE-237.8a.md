# BRIEF — Stone 237.8a — Arithmetic + Comparison HARD CUT under THE DECISION

**The big stone-block of arc 237's arithmetic tail.** Closes the cross-numeric
falsehood (`feedback_no_implicit_coercion`). Per `DESIGN-STONE-237.8.md` +
`probe_arc237_8a_no_implicit_coercion.rs` (both committed `4733b2a0`).

## THE DECISION (locked, the load-bearing axiom)

**No implicit numeric coercion across the substrate.** `(:wat::core::+ 1 2.0)`
→ ERROR. `(:wat::core::< 1 2.0)` → ERROR. Cross-type callers homogenize
explicitly (`(:wat::core::i64::to-f64 a)` then call same-type).

## The work — FIVE substrate files + probe un-ignore + consumer cascade

### 1. `wat/core.wat` — HARD CUT the 4 `define-dispatch` decls (lines 82-104)

DELETE all four:

```wat
(:wat::core::define-dispatch :wat::core::+'2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::+'2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::+'2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::+'i64'f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::+'f64'i64))
;; ... same shape for -'2, *'2, /'2
```

Tombstone comment in their place mirroring the 7c pattern: "arc 237 Stone
237.8a — `:wat::core::<op>'2` define-dispatch decls retired under THE DECISION
(`feedback_no_implicit_coercion`); same-type arithmetic routes directly through
per-Type leaves (`:wat::core::i64::+'2` / `:wat::core::f64::+'2`); cross-type is
rejected at check by `infer_arithmetic` (no longer f64-promoting). Mixed-type
Rust leaves (`+'i64'f64` etc.) deleted from substrate."

### 2. `src/check.rs` — TIGHTEN `infer_arithmetic` (lines 13211-13290)

The widest-contagion shape (f64 promotion when any arg is f64) becomes
same-type-only. Edit the 2+-ary logic at lines 13280-13289:

CURRENT:
```rust
let any_f64 = resolved.iter().any(|o| matches!(o, Some(t) if !is_i64(t) && is_numeric(t)));
let all_known_numeric = resolved.iter().all(|o| matches!(o, Some(t) if is_numeric(t)));
let ty = if all_known_numeric {
    if any_f64 { f64_ty } else { i64_ty }
} else {
    f64_ty
};
```

TIGHTENED (same-type-only):
```rust
// THE DECISION: no implicit numeric coercion. All numeric args MUST be the
// same type. Mixed (i64 + f64) is rejected at check; callers homogenize
// explicitly via `:i64::to-f64` (or vice versa).
let all_i64 = resolved.iter().all(|o| matches!(o, Some(t) if is_i64(t)));
let all_f64 = resolved.iter().all(|o| matches!(o, Some(t) if is_numeric(t) && !is_i64(t)));
let ty = match (all_i64, all_f64) {
    (true, _) => i64_ty,
    (_, true) => f64_ty,
    _ => {
        // Mixed (or unknown) — push TypeMismatch naming the first non-matching arg
        // (use the first non-i64-after-an-i64 as the canonical "the offending site").
        // Find the type of the first arg and complain about any subsequent arg that doesn't match.
        if let Some(Some(first_ty)) = resolved.first() {
            for (i, t_opt) in resolved.iter().enumerate().skip(1) {
                if let Some(t) = t_opt {
                    if !types_equal(t, first_ty) {
                        local_errors.push(CheckError::TypeMismatch {
                            callee: op.into(),
                            param: format!("#{}", i + 1),
                            expected: format_type(first_ty),
                            got: format_type(t),
                            span: args[i].span().clone(),
                        });
                        break;
                    }
                }
            }
        }
        f64_ty  // fallback; the error was pushed
    }
};
```

Helper needed: a `types_equal` function (probably already exists; grep
`fn types_equal\|fn type_equal\|fn same_type`). If not, write a trivial
`TypeExpr::Path` comparator. Or use `unify` with a fresh subst as a probe.

Also handle the 1-ary case (lines 13265-13278) — currently it's already
type-preserving (returns arg's type). KEEP unchanged.

ALSO update the docstring (lines 13189-13210) to reflect THE DECISION (no
f64-promotion language; same-type-only).

### 3. `src/check.rs` — TIGHTEN `infer_comparison` (lines 13128-13187)

DELETE the cross-numeric path at lines 13158-13160:

```rust
// DELETE THIS BLOCK:
if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
    return if local_errors.is_empty() { CheckResult::ok(bool_ty) } else { CheckResult::partial_with(bool_ty, local_errors) };
}
```

After deletion, the next path (the same-type-or-subtype unify check at lines
13168-13184) handles all comparisons uniformly. Numeric same-type (i64 vs i64,
f64 vs f64) succeeds via `unify`. Numeric mixed (i64 vs f64) fails unify AND
neither is a subtype of the other → pushes TypeMismatch.

ALSO update the docstring + the dispatch-site comment at check.rs:6473-6478
(remove "cross-numeric promotion" language; replace with "same-type or
subtype-related per THE DECISION").

### 4. `src/runtime.rs` — TIGHTEN `eval_arithmetic_variadic` (line 9910)

Read the function body and find where it dispatches per-pair through the
binary Dispatch. With the define-dispatch decls deleted, the Dispatch lookup
will fail. Replace the lookup with a direct routing on `Value` variants:

- Both args `Value::i64(_)` → call `:wat::core::i64::+'2` (or whichever op)
- Both args `Value::f64(_)` → call `:wat::core::f64::+'2`
- Mixed → teaching `RuntimeError::TypeMismatch` (this is the runtime guard
  paired with the check-time tightening; defense in depth)

The per-Type leaves (i64::+'2, f64::+'2) ARE registered substrate-side; route
to them via `env.lookup` + apply OR directly invoke their existing handler
functions.

### 5. `src/lexer.rs` — DELETE the `op'<t1>'<t2>` lexer entries (lines 1065-1066)

```rust
// DELETE:
":wat::core::op'f64'i64",
":wat::core::op'i64'f64",
```

These were the mixed-type leaf name patterns. With the leaves gone, the lexer
entries are dead.

### 6. Mixed-type Rust leaves — DELETE substrate registrations

The 8 mixed-type leaves (`+'i64'f64`, `+'f64'i64`, `-'i64'f64`, `-'f64'i64`,
`*'i64'f64`, `*'f64'i64`, `/'i64'f64`, `/'f64'i64`) need their substrate
registrations (in `register_builtins` of both src/check.rs and src/runtime.rs)
DELETED. Grep `'i64'f64\|'f64'i64` to find them; remove the registration
blocks; remove their handler function definitions if they're not called from
anywhere else.

### 7. `tests/probe_arc237_8a_no_implicit_coercion.rs` — un-ignore 3 rows

Remove the `#[ignore = "..."]` annotations on:
- `arith_i64_f64_mixed_rejected_at_check`
- `arith_f64_i64_mixed_rejected_at_check`
- `comparison_i64_f64_mixed_rejected_at_check`

After substrate edits, all 9 rows must PASS (the post-stone contract).

### 8. Consumer-sweep cascade (THE big unknown)

After the substrate tightens, `cargo build --release --tests --workspace`
will likely surface compile failures in any wat-side caller using cross-type
arithmetic. **This is the substrate-as-teacher cascade**
(`docs/SUBSTRATE-AS-TEACHER.md` + recovery doc FM 15). Each failure names
its call site.

Per `feedback_no_implicit_coercion`: the fix at every flagged site is
**explicit homogenization**. Cross-type `(+ a b)` becomes either
`(+ (:i64::to-f64 a) b)` or `(+ a (:f64::to-i64 b))` depending on the
intended type. **The substrate WILL NOT silently convert; the call site
must.**

Iterate: cargo test → read each error → migrate the cited site → cargo test
again. Repeat until 0 errors. Spot-checks suggest most callers are already
type-homogeneous (the cascade should be small).

## KEEP unchanged

- Per-Type leaves (`:wat::core::i64::+'2`, `:wat::core::f64::+'2`, etc.) —
  schemes, eval handlers, dispatch arms.
- Per-Type variadic wat fns (`:wat::core::i64::+`, `:wat::core::f64::+`,
  etc.) at `wat/core.wat:124-154` — defn wrappers around the per-Type leaves.
- `infer_polymorphic_holon_pair_to_f64` / `..._to_bool` / `..._to_path`
  (check.rs:13903+) — different polymorphism, NOT the same falsehood.
- `infer_polymorphic_time_arith` (check.rs:13511) — Duration math; out of
  scope.
- `is_numeric` (check.rs:13293) — STAYS as a helper (still useful for the
  "is this a numeric arg at all?" check before the type-equal check).
- `DispatchRegistry` + `src/dispatch.rs` — 237.8b's territory; don't touch
  this stone.
- All collection-op intrinsics (length, empty?, contains?, conj, get, assoc)
  — 237.7 done; untouched.

## Verify (RAW commands — no wrapper scripts)

Run these as SEPARATE simple commands, one per line:

- `cargo build --release -p wat` → 0 errors
- `cargo test --release --test probe_arc237_8a_no_implicit_coercion` →
  `9 passed; 0 failed; 0 ignored` (post-un-ignore)
- `cargo build --release --tests --workspace` → 0 errors (this is THE
  consumer-sweep cascade endpoint — every cross-type call site has been
  migrated)
- `cargo test --release --lib -p wat` → `834+ passed; 0 failed` (lib
  baseline; may DROP if any lib test asserted cross-type arithmetic worked
  — those tests need migration too, applying the same THE DECISION
  homogenization)

Do NOT invoke `./scripts/green-gate.sh` — wrapper scripts get denied; use the
raw commands above.

## STOP triggers (REJECTION — surface, do not work around)

- If you find yourself ADDING back any implicit-coercion path (e.g., "if
  mixed, promote to f64 as fallback") → STOP. THE DECISION rejects this.
- If you touch `infer_polymorphic_holon_pair_*` or
  `infer_polymorphic_time_arith` bodies → STOP. Out of scope.
- If you delete `src/dispatch.rs` or any `DispatchRegistry` use site → STOP.
  That's 237.8b's job; this stone leaves the registry as-is (just with
  zero tenants).
- If you delete per-Type leaves (`:wat::core::i64::+'2` etc.) → STOP.
  Those are the irreducible primitives; they STAY.
- If the consumer-sweep cascade requires touching the lab/holon-rs/anything
  outside wat-rs/ → STOP. holon-rs is FROZEN (STOP-5). The lab isn't in
  scope; if a lab consumer needs migration, surface as a follow-up.
- If you re-add `#[ignore]` to the probe rows to "make the build green" →
  STOP. Honest red beats dishonest green.
- If you discover that holon-pair OR time-arith handlers also have the
  cross-type falsehood (i.e., they DO match arc 237.8's pattern) → STOP and
  surface in the SCORE. Re-brief decides whether to bundle or split.

## Definition of done

- All 9 probe tests green; 0 ignored.
- Workspace test-build 0 errors (consumer cascade fully resolved).
- Lib 834+/0 (or honestly-explained drop if any lib test asserted the old
  falsehood).
- `wat/core.wat` no longer has the 4 `define-dispatch` decls; tombstone in
  place.
- `src/check.rs` has tightened `infer_arithmetic` + tightened
  `infer_comparison`; no cross-numeric path remains in either.
- `src/runtime.rs` has tightened `eval_arithmetic_variadic`.
- `src/lexer.rs` lines 1065-1066 deleted.
- 8 mixed-type leaves' substrate registrations DELETED; their handler
  functions DELETED (if dead).
- Probe's three `#[ignore]` annotations removed.
- DispatchRegistry untouched (8b's job).
- Only the scoped substrate files + the probe + the SCORE + any
  consumer-sweep .wat/.rs sites touched.
- Write `SCORE-STONE-237.8a.md` (sibling); do NOT commit (orchestrator
  scores + commits).
- The SCORE explicitly enumerates the consumer-sweep cascade: which files
  were touched, what coercion was added at each site, what the runtime
  count delta was.
