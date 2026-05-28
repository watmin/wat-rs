# Stone 237.8 — Arithmetic + Comparison HARD CUT under THE DECISION (no implicit coercion); arc-146 Dispatch HARD CUT

**Closes the arithmetic tail of arc 237** (and arc 146 with it). After 237.7
complete (all collection-op intrinsic surfaces evacuated), arithmetic is the
LAST `define-dispatch` tenant of the arc-146 `DispatchRegistry`. Once it
evacuates, the registry HARD CUTs as well.

## THE DECISION (locked, from memory + cliffnotes)

`feedback_no_implicit_coercion`: **no implicit numeric coercion across the
entire substrate**. `(:wat::core::+ 1 2.0)` → ERROR. Homogenize explicitly
(`1.0`, or `(:i64::to-f64 a)`). typeunion → consumed by DISCRIMINATION
(`is-X?`); arithmetic → concrete-per-type; **the two never touch**.

This stone applies THE DECISION to BOTH arithmetic AND comparison families
(the cliffnotes "Headline state" originally listed comparison as a separate
sub-family; empirically it's the same falsehood — `infer_comparison:13158`
accepts cross-numeric `(<, i64, f64)` silently, identical pattern to
`infer_arithmetic`'s f64-promotion).

## Crawl (ground truth, 2026-05-27, HEAD `169c5e07`)

### The three-layer architecture (per arc 148 slice 4, wat/core.wat:57-80)

| Layer | Surface | Today | Under THE DECISION |
|---|---|---|---|
| **1** — Polymorphic variadic | `:wat::core::+`/`-`/`*`/`/` (bare) | `infer_arithmetic` (custom; widest-contagion to f64) + `eval_arithmetic_variadic` left-fold | **TIGHTENED**: require same-type at check + runtime; STAYS as a custom-inference handler (the variadic ergonomic surface is honest under same-type discipline) |
| **2** — Binary Dispatch | `:wat::core::+'2` etc. (4 decls; 4 arms each) | arc-146 `define-dispatch` with mixed-type arms (i64+f64 → `+'i64'f64`; f64+i64 → `+'f64'i64`) | **DELETED**: the entity exists ONLY to host mixed-type arms; same-type arms collapse to direct per-Type leaf calls. Decls go; the Dispatch entity becomes a 0-tenant registry. |
| **3** — Per-Type Rust primitives + mixed-type leaves | `:wat::core::i64::+'2`, `:wat::core::f64::+'2` (per-Type) + `:wat::core::+'i64'f64`, `:wat::core::+'f64'i64` etc. (mixed) | both registered substrate-side; lexer recognizes both name patterns (`src/lexer.rs:1065-1066`) | per-Type leaves **KEEP** (irreducible); mixed-type leaves **DELETED** (8 total: 4 ops × 2 directions); lexer entries for `op'<t1>'<t2>` **DELETED** |

### Comparison family — same falsehood at `infer_comparison`

**6 ops route to `infer_comparison`** (check.rs:6485): `=`, `not=`, `<`, `>`,
`<=`, `>=`. Today (check.rs:13158):

```rust
if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
    return CheckResult::ok(bool_ty);  // accepts (i64, f64) silently
}
```

Under THE DECISION: DELETE the cross-numeric path; fall through to the
unify-or-subtype check. Numeric comparison becomes same-type-only at check
(runtime already same-type per the per-Type leaves; no change needed there).

### Handlers NOT in scope (out of 237.8)

`infer_polymorphic_holon_pair_to_f64` (check.rs:13903), `..._to_bool` (14120),
`..._to_path` (14179), `infer_polymorphic_time_arith` (13511) — these are
NOT the same falsehood. They're custom inference for legitimate polymorphism
(HolonAST vs Vector input shapes; time-vs-duration arithmetic semantics).
**They don't tenant the DispatchRegistry.** Out of arc 237.8's scope per the
one-canonical-path doctrine — don't touch what no caller demands change of.
If the audit surfaces a problem with them later, they get their own arc.

### DispatchRegistry tenancy (verified via grep)

```bash
grep "define-dispatch" wat/core.wat  # only 4 decls: +'2, -'2, *'2, /'2
```

ALL `DispatchRegistry` tenants are arithmetic. Once 237.8a evacuates the 4
decls, the registry is empty; the guard at check.rs:5460
(`if let Some(reg) = env.dispatch_registry() { ... }`) returns None always.
Atomic HARD CUT.

### Consumer-sweep audit (cross-type call-site risk)

~20 files use bare `:wat::core::+`/`-`/`*`/`/`. Spot-checks (`(:wat::core::-
0.0 ratio)` = f64+f64, `(:wat::core::+ (...) 1)` = i64+i64, `(:wat::core::*
2.0 pi)` = f64+f64) suggest **most callers are already type-homogeneous**.
The substrate-as-teacher cascade (`docs/SUBSTRATE-AS-TEACHER.md`) handles any
cross-type stragglers: substrate tightens → consumers fail with named-site
diagnostics → migrate by adding explicit coercion (`:i64::to-f64`) at the
flagged site. Per `feedback_no_implicit_coercion`: this is the explicit
homogenization the doctrine demands.

## Slicing

**237.8a — Arithmetic + Comparison HARD CUT + tighten** (the big stone, the
behavior change). Substrate change + consumer-sweep cascade. Bundles
arithmetic + comparison because they share THE DECISION rationale + the
fixes are independent (no ordering dependency).

**237.8b — DispatchRegistry HARD CUT** (mechanical cleanup after 8a).
DELETE `src/dispatch.rs` + `dispatch_registry` field on `CheckEnv` +
`SymbolTable` + `set_dispatch_registry` / getter + the guard at
`check.rs:5460`. No behavior change (8a evacuated all tenants); workspace
should compile + tests pass identically.

**237.9 INSCRIPTION** (closure). Folds arc 146 (DispatchRegistry retired) +
arc 148 (arithmetic family migrated) + arc 237 (umbrella). USER-GUIDE
sections: base-vs-holonic records (owed from arc 234.7); THE DECISION as
canonical reference; the four families' final shapes.

### Why bundle arithmetic + comparison in 8a (not separate stones)

- They share THE DECISION rationale (no implicit numeric coercion). Splitting
  fragments the inscription.
- Comparison fix is TINY (delete 3 lines at `infer_comparison:13158-13160`);
  bundling adds < 5 min to 8a's runtime.
- Same-day landing keeps the doctrine atomic in the commit history.
- The consumer-sweep cascade for arithmetic SUBSUMES comparison's (any
  cross-type call site in `(+ ...)` is likely paired with a `(< ...)`).

### Why split 8a + 8b (not bundle)

- 8a has a real behavior change + consumer sweep (unbounded cascade size).
- 8b is mechanical substrate-internal cleanup (bounded; no behavior change).
- Atomic commits review cleaner if separated.
- If 8a's cascade is bigger than predicted, 8b doesn't need re-briefing.

## FM-2-bis probe (for 8a) — what it must prove

`tests/probe_arc237_8a_no_implicit_coercion.rs`. Disconfirming AT HEAD
`169c5e07`+ (cross-type passes today); regression contract for same-type +
the new rejections post-stone.

| # | Test | At HEAD | Post-8a |
|---|---|---|---|
| 1 | `arith_i64_same_type_works` | PASS | PASS (regression) |
| 2 | `arith_f64_same_type_works` | PASS | PASS (regression) |
| 3 | `arith_i64_f64_mixed_rejected_at_check` | **PASS** (today returns 3.0) → must flip | flips to: result.is_err() |
| 4 | `arith_f64_i64_mixed_rejected_at_check` | **PASS** (today returns 3.0) → must flip | flips to: result.is_err() |
| 5 | `arith_variadic_same_type_three_args_works` | PASS | PASS (regression — variadic shape preserved) |
| 6 | `comparison_i64_same_type_works` | PASS | PASS (regression) |
| 7 | `comparison_f64_same_type_works` | PASS | PASS (regression) |
| 8 | `comparison_i64_f64_mixed_rejected_at_check` | **PASS** (today `(< 1 2.0)` returns true silently) → must flip | flips to: result.is_err() |
| 9 | `comparison_string_same_type_works` | PASS | PASS (regression — non-numeric same-type unaffected) |

Tests 3, 4, 8 are the disconfirming load-bearing rows. They use the
`#[ignore]` pattern from 7c — at HEAD they would PASS (returning 3.0 / 3.0 /
true) which is the falsehood; post-stone they must error. The `#[ignore]`
annotation on these rows acknowledges the test ASSERTION (assert result is
error) doesn't hold at HEAD; sonnet's stone work removes the annotations.

Commit the probe with sub-DESIGN BEFORE the BRIEF. Sonnet mirrors it.

## What this stone does NOT do

- Touch `infer_polymorphic_holon_pair_*` / `infer_polymorphic_time_arith` (out
  of scope; different polymorphism, no falsehood).
- Mint defclauses for arithmetic (per the original cliffnotes Headline state).
  After analysis, single-impl per-type via direct per-Type leaves is simpler
  than wrapping each in a defclause; defclause was the right answer when we
  needed multi-impl dispatch, but THE DECISION removes the cross-type arms,
  so each per-Type leaf is single-impl → just a `defn` or a substrate primitive.
- Retire the variadic ergonomic surface (`:wat::core::+` etc.). It STAYS as a
  custom-inference handler under same-type discipline (per Option A in the
  consideration below).

## Considered + rejected: Option B (delete the bare variadic surface)

Considered: DELETE `:wat::core::+` (etc.) entirely; force callers to use
`:wat::core::i64::+` or `:wat::core::f64::+` explicitly. **Rejected** because:
- Sweeping consumer change (~20+ files; every call site must choose a type).
- The ergonomic variadic surface IS honest under same-type discipline (no
  more masking; the type is just the type).
- The cliffnotes "Headline state" + DESIGN-STONE-237.7-intrinsic-kill.md both
  preserve "polymorphic variadic at `:wat::core::<v>` (bare name) STAYS".

Option A (tighten the variadic + keep the surface) IS the chosen path.

## Constraints (for 8a)

- Edits in `src/check.rs` + `src/runtime.rs` + `wat/core.wat` +
  `src/lexer.rs` + `tests/probe_arc237_8a_no_implicit_coercion.rs` only.
  Plus the consumer-sweep cascade (whichever sites the substrate-as-teacher
  surfaces).
- NO holon-rs. NO touch of holon-pair / time-arith handlers. NO touch of
  per-Type leaves. NO `DispatchRegistry` deletion (237.8b). NO touch of
  collection-op intrinsics (237.7 done).
- Green-gate (momentary): `cargo test --release --lib -p wat` (834+/0) +
  `cargo build --release --tests --workspace` (0 errors). **Raw cargo
  commands, NO wrapper scripts** (per `feedback_sonnet_bash_firewall`).
- Probe is the regression contract: 9/9 post-stone (3 ignored→un-ignored).
- HARD CUT discipline — DELETE the 4 dispatch decls; DELETE the 8 mixed
  leaves; DELETE the cross-numeric path in comparison. No shims, no "if mixed
  fallback to promotion."
- The substrate-as-teacher cascade is acknowledged scope — sonnet iterates
  the failures (each error names the next site to fix via explicit
  `:to-f64` coercion).

## Open question — wat-tests/time.wat

`wat-tests/time.wat:153-351` has dozens of `(:wat::core::-` and
`(:wat::core::*)` calls in Duration math. If any are cross-type (Duration vs
i64 ms, etc.), they fall under the time-arith handler (out of scope), NOT
the arithmetic handler — verify in the BRIEF. The BRIEF instructs sonnet to
grep `infer_polymorphic_time_arith` to see what ops it covers and confirm the
time.wat calls route there, not to `infer_arithmetic`.
