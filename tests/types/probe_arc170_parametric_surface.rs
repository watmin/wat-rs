//! Arc 170 C2 — wards parametric surfaces: a `defsurface :Name<T>` method whose return type
//! `:T` resolves to the concrete type bound by a satisfier's `extend-type :Satisfier
//! :Name<Concrete>`, both at the call site (return-type resolution) and against the satisfier's
//! own method body (receiver-satisfaction). Commits `7d8e3034` ("170 C2: parametric defsurface —
//! surface-level <T> flows through extend-type + call-site dispatch") and `b2360c7a` ("170 C2:
//! fix receiver-satisfaction gap for parametric surfaces with embedded returns") shipped this
//! substrate capability with no committed test — this promotes the proven
//! scratchpad/probe-c2-parametric-surface{,-neg}.wat probes into a durable ward, plus a third
//! soundness fixture proving a mistyped satisfier is rejected, not silently accepted.
//!
//! 1. `parametric_surface_return_resolves_to_satisfier_type` — positive: `Holds<T>`'s `get`
//!    returns `:T`; `IntBox` satisfies `Holds<i64>`; `(Holds/get b)` evaluates to `Value::i64(42)`.
//! 2. `parametric_surface_return_is_typed_not_any` — negative: ascribing the resolved return to
//!    `:wat::core::String` (instead of the correct `i64`) is a located `TypeMismatch`.
//! 3. `parametric_surface_rejects_mistyped_satisfier` — negative (soundness): `BadBox` claims
//!    `Holds<i64>` but its `get` returns a `String` field — a located `ReturnTypeMismatch`.

use wat::ast::WatAST;
use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

#[test]
fn parametric_surface_return_resolves_to_satisfier_type() {
    let world = startup_from_file("tests/types/probe_arc170_parametric_surface.wat")
        .expect("startup should succeed: parametric defsurface return resolves per satisfier");
    // `:probe::resolve` (a non-main defn — the fixture carries no `:user::main`, per the arc-170
    // `[] -> :nil` / UselessMain wall) returns `(Holds/get b)`, whose resolved return type is i64.
    // Eval it via a PROGRAMMATICALLY built call AST (not a `parse_one!`-string), so this test
    // inlines no wat form (no_inlined_wat clean).
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::resolve".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("resolve raised: {e:?}"))
        .value_owned();
    match got {
        Value::i64(42) => { /* Holds<T>/get resolved T = i64 per IntBox's extend-type */ }
        other => panic!(
            "expected Value::i64(42): the parametric surface's `-> :T` should resolve to i64 per \
             the IntBox satisfier. got {other:?}"
        ),
    }
}

#[test]
fn parametric_surface_return_is_typed_not_any() {
    // The parametric surface's resolved return type is genuinely i64 (not bare/any): ascribing
    // it to String is a located TypeMismatch (expected String, got i64) at the `ann-form` site
    // (probe_arc170_parametric_surface.wat.bad:16, the `bad` binding). Asserted STRUCTURALLY on
    // the error enum (not a `contains` substring, not an EDN golden) so the expected/got are the
    // exact bare-keyword type strings.
    let err = startup_from_file("tests/types/probe_arc170_parametric_surface.wat.bad")
        .expect_err("wrong ascription must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    match &errs[0].kind {
        CheckErrorKind::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, ":wat::core::String");
            assert_eq!(got, ":wat::core::i64");
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn parametric_surface_rejects_mistyped_satisfier() {
    // Soundness: BadBox claims Holds<i64> via extend-type, but its `get` body returns its own
    // String field — a located ReturnTypeMismatch (probe_arc170_parametric_surface_soundness.wat.bad:13,
    // the `get` method body), proving the receiver-satisfaction check (commit b2360c7a) actually
    // verifies the satisfier's method body, not just its signature. Asserted STRUCTURALLY on the
    // error enum (not a `contains` substring, not an EDN golden).
    let err = startup_from_file("tests/types/probe_arc170_parametric_surface_soundness.wat.bad")
        .expect_err("mistyped satisfier must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    match &errs[0].kind {
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. } => {
            assert_eq!(function, ":probe::BadBox/get");
            assert_eq!(expected, ":wat::core::i64"); // Holds<i64> wants i64
            assert_eq!(got, ":wat::core::String"); // BadBox/get returns its String field
        }
        other => panic!("expected ReturnTypeMismatch, got {other:?}"),
    }
}
