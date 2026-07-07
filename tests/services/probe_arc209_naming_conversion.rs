//! Arc 209 — PascalCase ⇄ kebab-case naming-conversion tooling (both directions).
//!
//! defservice derives kebab fn names from PascalCase op keywords; bare `to-lowercase` mis-names
//! multi-word ops (`:GetObject` → `getobject`, not `get-object`). This stone builds the full
//! converter — `pascal->kebab` (Rust intrinsic, macro-needed), `to-uppercase` (Rust primitive),
//! `kebab->pascal` (wat helper) — and threads the forward direction into defservice.
//!
//! RED at HEAD: the three ops don't exist; defservice derives `getobject-request` (so the
//! `:my::svc/get-object-request` constructor is unresolved). GREEN once the tooling ships + is
//! threaded.
//!
//! Run: cargo test --release -p wat --test probe_arc209_naming_conversion

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn s(world: &wat::freeze::FrozenWorld, call: &str) -> String {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new()).map(|tv| tv.value_owned()).expect("eval") {
        Value::String(v) => (*v).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn pascal_kebab_both_directions_and_roundtrip() {
    // Wat source lives in the co-located fixture: probe_arc209_naming_conversion.wat
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(s(&world, r#"(:user::p2k "GetObject")"#), "get-object");
    assert_eq!(s(&world, r#"(:user::p2k "Get")"#), "get");
    assert_eq!(s(&world, r#"(:user::up "abc")"#), "ABC");
    assert_eq!(s(&world, r#"(:user::k2p "get-object")"#), "GetObject");
    // bijection on the disciplined subset (one capital per word)
    assert_eq!(s(&world, r#"(:user::roundtrip "GetObject")"#), "GetObject");
}

// Arc 278 S4c: :ops RETIRED — the service now wears a surface (:satisfies + :impls). A
// MULTI-WORD op (`get-object`) must round-trip kebab<->pascal correctly through the surface
// path: the KEBAB client method `:my::svc/get-object` (op-str verbatim) AND the pascal record /
// Op names `:my::Svc::GetObjectRequest` / `:my::Svc::Op::GetObject` (kebab->pascal-in). If the
// multi-word conversion were broken ("get-object" -> "Getobject"), the generated req-ty would
// not resolve to the user-declared record and startup would fail. Running the service
// end-to-end through `:my::svc/get-object` therefore proves the multi-word derivation.
// arc 291 4b-ii: State is a defstruct; :durable mints ::Record; start takes ::Record.
#[test]
fn defservice_multiword_op_derives_kebab_names() {
    // Wat source: probe_arc209_naming_conversion_svc.wat
    let world = startup_from_file("tests/services/probe_arc209_naming_conversion_svc.wat")
        .expect("startup (defservice multi-word op round-trips kebab<->pascal in the surface path)");
    let ast = wat::parse_one!("(:user::req-id)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("req-id raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: the multi-word op `get-object` must wire up the KEBAB client method \
         `:my::svc/get-object` + pascal `:my::Svc::GetObjectRequest`/`Op::GetObject` \
         (kebab->pascal-in), not `getobject`/`Getobject`; got {got:?}"
    );
}
