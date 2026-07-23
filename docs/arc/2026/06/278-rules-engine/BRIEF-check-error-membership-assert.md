# BRIEF — Strike M: measure the check-error SET as a set (data equality, not position)

> **Tier:** sonnet shadowdancer. **Arc:** 278 (a floor-determinism fix surfaced by the item-c thread).
> **HEAD:** `25e316f0` (Strike A's Failure-constructor edits are uncommitted in the tree — DISJOINT from
> your files; leave them untouched).

## Why (one paragraph)

`check_program` returns `CheckErrors(pub Vec<CheckError>)` — a **set** of findings whose order carries no
meaning and is **per-process nondeterministic** (Rust `HashMap`/`RandomState` reseeds each process). 9 test
sites assert `match &errs[0].kind { … }` — **positional indexing into that unordered set.** When the form
under test emits more than one error, `errs[0]` is a coin-flip: `probe_arc170_c2_mixed_macro::mixed_via_macro_swap_is_compile_error`
flakes ~50% (proven: 6 isolated runs P,P,F,F,F,P). The measurement is wrong — the correct assertion is
**data equality on set membership**: *the expected error is a member of the set*, order-independent. This
strike adds one canonical membership assert and routes all 9 sites through it. **No checker changes** — the
checker isn't the bug; the tests are measuring a set as a sequence.

## The one contract decision — a canonical `assert_check_error_present!` macro

Add it to `src/lib.rs`, next to `assert_edn_eq!` (:176) / `assert_edn_matches_file!` (:236), `#[macro_export]`
so integration tests reach it as `wat::assert_check_error_present!`. It asserts that **some** `CheckError`
in the set matches a given `CheckErrorKind` pattern + (optional) guard predicate, and on failure dumps the
**whole set** as EDN (`CheckErrors` Debug = EDN). Shape (adapt to the exact `CheckError`/`CheckErrorKind`
types you read in `src/check/error.rs:24,56,793`):

```rust
#[macro_export]
macro_rules! assert_check_error_present {
    ($errs:expr, $pat:pat $(if $guard:expr)? $(,)?) => {{
        let __errs = &$errs;
        assert!(
            __errs.iter().any(|__e| matches!(&__e.kind, $pat $(if $guard)?)),
            "no check error matched `{}`; errors were:\n{:?}",
            stringify!($pat $(if $guard)?),
            $crate::check::error::CheckErrors(__errs.to_vec()),   // if CheckError: !Clone, format each via Display instead
        );
    }};
}
```

## The conversion — every `errs[0]` positional match → membership (EXACT structure, order-independent)

The **9 sites** (7 files), each `match &errs[0].kind { <Variant>{fields} => { assert_eq!(…); … } other => panic! }`:

- `tests/services/probe_arc170_c2_mixed_macro.rs:70`
- `tests/services/probe_arc170_wrong_service_compile_error.rs:44` **and** `:61`
- `tests/services/probe_arc170_c2_d_bodiless_edge.rs:43`
- `tests/services/probe_arc170_w2a_kwargs_check_mint.rs:32`
- `tests/services/probe_arc278_journal_surface.rs:62`
- `tests/types/probe_arc170_parametric_surface.rs:59` **and** `:80`
- `tests/types/probe_arc170_parametric_surface_param.rs:44`

For each, fold the site's `assert_eq!(expected, "…")` / `assert_eq!(got, "…")` (etc.) into the macro's
**guard predicate** as exact `==` comparisons, so the assertion is: *the set contains an error of this
kind with exactly these fields.* Example (`probe_arc170_c2_mixed_macro.rs:70`):

```rust
assert_check_error_present!(errs,
    wat::check::error::CheckErrorKind::TypeMismatch { expected, got, .. }
        if expected == ":wat::capability::TypedCapability<probe::S1::Op,probe::S1::Reply>"
        && got == ":probe::s2'::Handle");
```

**Preserve each site's EXACT expected values** — this is data equality on structure, NOT a loosening. Do
NOT replace `expected == "…"` with a substring/`contains` check (that is the loose-assert anti-pattern the
arc already cut). Same structure asserted, only the *positional* index removed. A site that today asserts
several fields → put all of them in the guard with `&&`.

## STOP triggers

- **STOP-0:** you find yourself editing `src/check.rs` / `src/check/error.rs` *logic* (e.g. sorting the
  error Vec by span) — STOP. Sorting the producer is treating the symptom; the fix is the test measuring the
  set as a set. The only `src/` edit permitted is **adding the one macro to `src/lib.rs`** (+ a `to_vec`/
  Clone accommodation on `CheckError` in `check/error.rs` *only if* the macro's message needs it and it's a
  trivial `#[derive(Clone)]`; if that derive is non-trivial, fall back to formatting each error via Display
  and touch nothing else).
- **STOP-1:** a converted site's original assertion was doing something other than exact field equality
  (e.g. it already searched, or asserted a count) — STOP on that site, report it; don't force it into the
  macro.
- **STOP-2:** the ex-flaky test does NOT go deterministically green after conversion — STOP and report; it
  would mean the flake has a second cause beyond `errs[0]`.

## Verify (weigh by your own re-run)

1. `cargo nextest run --release` builds clean (the new macro compiles; all 9 sites compile).
2. **Determinism proof** — run the ex-flaky test **20×** and confirm ALL green:
   `for i in $(seq 1 20); do cargo nextest run --release 'probe_arc170_c2_mixed_macro::mixed_via_macro_swap_is_compile_error' 2>&1 | grep -E "Summary"; done`
   — every line must read `1 passed`. (Before this strike it was ~50% `1 failed`.)
3. **Floor:** `cargo nextest run --release` — READ THE SUMMARY LINE yourself; the previously-1-failed floor
   must now be `0 failed` (modulo the by-design `#[ignore]`'d self_scheduling ×2). Run it 2×; both `0 failed`.

## Deliverable

The `assert_check_error_present!` macro in `src/lib.rs` + all 9 sites converted to membership. Report: (1)
the macro's final form; (2) the 20× determinism result (all green); (3) two full-floor Summary lines (both
0-failed); (4) `git diff --stat` (the macro + the ≤7 test files; Strike A's `runtime.rs`/`service.wat`/
`spawn.wat`/fixture edits untouched by you). Do NOT commit — leave it for the orchestrator to weigh.

## Blast radius

`src/lib.rs` (one new macro) + the 7 test `.rs` files (9 sites). NO checker logic. NO `wat/`. NO touching
Strike A's uncommitted edits. Scratch logs → `/tmp/claude-scout/`.
