//! FM 2-bis probe — arc 251 Stone 251.4b: `(ann-form expr type)` ascription.
//!
//! core.typed's `ann-form` is a CHECKED, type-erased identity: `(ann-form e T)`
//! asserts `e`'s type is assignable to `T` (the form's type becomes `T`), and at
//! runtime evaluates `e` and returns its value. Head: `:wat::core::ann-form`
//! (the symbol surface `wat.core/ann-form` resolves via the normalize-layer).
//!
//! HEAD-disconfirmation:
//! - C01: `(:wat::core::ann-form 41 :wat::core::i64)` type-checks AND evaluates to 41
//!   ⇒ FAILS at HEAD (`:wat::core::ann-form` is not a registered form).
//! - C02: `(:wat::core::ann-form 42 :wat::core::String)` is REJECTED (42 is i64, not
//!   String) — proves the ascription is CHECKED, not a pass-through no-op.
//! - C03: `(:wat::core::ann-form 41 wat.type/i64)` checks clean — the type slot reuses
//!   `parse_type_node`, inheriting the `wat.type/` surface (251.2a).
//!
//! Post-251.4b: C01 + C03 pass (Ok); C02 stays Err (the rejection is the point).
//!
//! Run: `cargo test --release --test probe_arc251_stone4b_ann_form`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Freeze + check `decls` (plus a legacy-spelled main). Ok(()) iff it type-checks.
fn checks(decls: &str) -> Result<(), String> {
    let src = format!("{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

/// Eval `body` (declared `-> :wat::core::i64`) via `:user::compute`; return the i64.
fn eval_i64(body: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

// ─── C01: THE GAP — ann-form checks AND evaluates to the inner value ────────────

#[test]
fn contract_01_ann_form_checks_and_evaluates() {
    assert_eq!(
        eval_i64("(:wat::core::ann-form 41 :wat::core::i64)"),
        Ok(41),
        "(ann-form 41 :i64) must type-check and evaluate to 41 (type-erased identity)"
    );
}

// ─── C02: the ascription is CHECKED (a mismatch is rejected) ────────────────────

#[test]
fn contract_02_mismatched_ascription_rejected() {
    assert!(
        checks("(:wat::core::defn :user::f [] -> :wat::core::String \
                  (:wat::core::ann-form 42 :wat::core::String))")
            .is_err(),
        "(ann-form 42 :String) must be REJECTED — 42 is i64, not String (ascription is checked)"
    );
}

// ─── C03: the type slot reuses parse_type_node (wat.type/ surface) ──────────────

#[test]
fn contract_03_ann_form_accepts_wat_type_surface() {
    assert!(
        eval_i64("(:wat::core::ann-form 41 wat.type/i64)").is_ok(),
        "(ann-form 41 wat.type/i64) must check — the type slot accepts the wat.type/ surface"
    );
}
