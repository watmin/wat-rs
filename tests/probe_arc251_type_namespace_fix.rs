//! Strike (examinare probe) — 4.0: the faithful type-NAMESPACE fix (intueri-named).
//!
//! `type_expr_to_faithful_watast` (edn_shim.rs) flattens EVERY named type to `wat.type/<last-segment>`
//! via `rsplit("::").next()`. That LIES for user/library types: two distinct types with the same
//! final segment collapse to one name. The intueri-named scheme:
//!   - core/built-in types  -> flat `wat.type/Name`  (FQDN `wat::core::X` OR a BARE_PRIMITIVE :i64/:String/…)
//!   - user/library types   -> preserve namespace via `wat_keyword_to_clojure_symbol`
//!   - bare Uppercase non-primitive -> type-var (bare symbol)
//!
//! Asserts the DESIRED end-state so the build turns these green WITHOUT editing the probe.
//! GREEN at HEAD: C01/C02/C07 (preservation). RED at HEAD: C03 (`:String` mis-rendered bare),
//! C04/C05 (user-type COLLISION), C06 (user type flattened into wat.type/).
//!
//! Run: `cargo test --release --test probe_arc251_type_namespace_fix`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_string(body: &str) -> Result<String, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::String {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

fn to_type(kw: &str) -> Result<String, String> {
    eval_string(&format!(
        "(:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node \"{kw}\")))"
    ))
}

#[test]
fn c01_core_fqdn_scalar_stays_wat_type() {
    // Preservation — green at HEAD.
    assert_eq!(to_type(":wat::core::i64"), Ok("wat.type/i64".into()));
    assert_eq!(to_type(":wat::core::String"), Ok("wat.type/String".into()));
}

#[test]
fn c02_core_parametric_stays_wat_type() {
    // Preservation — green at HEAD.
    assert_eq!(
        to_type(":wat::core::Vector<wat::core::i64>"),
        Ok("(wat.type/Vector wat.type/i64)".into())
    );
}

#[test]
fn c03_bare_legacy_primitive_renders_core() {
    // RED at HEAD: `:String` is bare + Uppercase -> currently mis-rendered as bare `String`
    // (looks like a type-var). It is a BARE_PRIMITIVE -> must be `wat.type/String`.
    assert_eq!(to_type(":i64"), Ok("wat.type/i64".into()));
    assert_eq!(to_type(":String"), Ok("wat.type/String".into()));
    assert_eq!(to_type(":bool"), Ok("wat.type/bool".into()));
}

#[test]
fn c04_user_type_preserves_namespace() {
    // RED at HEAD: currently flattened to `wat.type/Req`.
    assert_eq!(
        to_type(":wat::kernel::services::StdErrService::Req"),
        Ok("wat.kernel.services.StdErrService/Req".into())
    );
}

#[test]
fn c05_distinct_user_types_do_not_collide() {
    // THE load-bearing disconfirmer. RED at HEAD: both -> `wat.type/Req`.
    let a = to_type(":wat::kernel::services::StdErrService::Req");
    let b = to_type(":wat::kernel::services::StdInService::Req");
    assert!(a.is_ok() && b.is_ok(), "both must render: {a:?} {b:?}");
    assert_ne!(a, b, "distinct types must NOT render to the same faithful name (collision)");
}

#[test]
fn c06_user_type_two_segment_preserves_namespace() {
    // RED at HEAD: currently `wat.type/HolonAST`.
    assert_eq!(to_type(":wat::holon::HolonAST"), Ok("wat.holon/HolonAST".into()));
}

#[test]
fn c07_type_var_stays_bare() {
    // Preservation — green at HEAD. Uppercase, no `::`, NOT a primitive -> bare var.
    assert_eq!(to_type(":T"), Ok("T".into()));
    assert_eq!(to_type(":K"), Ok("K".into()));
}
