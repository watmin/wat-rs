//! Arc 265 — the namespace-scoped acronym registry.
//!
//! Restores the casing PascalCase⇄kebab can't carry (AWS `WebACL ⇄ web-acl ⇄ WebACL`). The registry
//! is keyed by namespace: `my::ns` owns its acronyms; no entry → default plain conversion. Threaded
//! into defservice's op-name derivation (the expand-time consumer).
//!
//! Positive test uses the co-located sibling fixture (CONV program):
//!   tests/macros/probe_arc265_acronym_registry.wat
//!
//! Second test uses the explicit SVC fixture (separate program):
//!   tests/macros/probe_arc265_acronym_registry_svc.wat
//!
//! Run: cargo test --release -p wat --test probe_arc265_acronym_registry

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn eval_string(world: &wat::freeze::FrozenWorld, call: &str) -> String {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval")
    {
        Value::String(v) => (*v).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn namespace_scoped_acronym_conversion_restores_casing() {
    let world = startup_beside(file!()).expect("startup");
    // my::aws has ACL declared → forward emits the acronym as one segment, reverse restores it.
    assert_eq!(eval_string(&world, r#"(:user::fwd "CreateWebACL")"#), "create-web-acl");
    assert_eq!(eval_string(&world, r#"(:user::rev "create-web-acl")"#), "CreateWebACL");
    assert_eq!(eval_string(&world, r#"(:user::roundtrip "CreateWebACL")"#), "CreateWebACL");
    // :other::ns has NO acronyms → default: ACL is not restored.
    assert_eq!(eval_string(&world, r#"(:user::rev-default "create-web-acl")"#), "CreateWebAcl");
}

// The defsurface/defservice pipeline consults its namespace acronyms at EXPAND/REGISTER time —
// proves declare-acronyms populated the registry before the surface's S1 protocol synthesis and
// the service's :impls op-name derivation ran, and that BOTH restored the acronym casing.
// arc 278 S4c: the SVC fixture is now a `:satisfies` service whose Peer surface owns its
// `:messages` protocol; the kebab method `create-web-acl` must synthesize `::Op::CreateWebACL`
// (ACL declared for the surface `:my::aws::Waf`) — not `::Op::CreateWebAcl`. `(:user::req-n)`
// constructs + matches that exact synthesized variant, so it only resolves when the acronym
// carried through S1's `synthesize_surface_protocol` (mirroring how `:impls` threads the registry).
#[test]
fn defservice_consults_its_namespace_acronyms_at_expand_time() {
    let world = startup_from_file("tests/macros/probe_arc265_acronym_registry_svc.wat")
        .expect("startup (:satisfies surface synthesizes ::Op::CreateWebACL with ACL declared)");
    let ast = wat::parse_one!("(:user::req-n)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("req-n raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected 7: the surface `:my::aws::Waf` (ACL declared BEFORE it registers) must synthesize \
         `:my::aws::Waf::Op::CreateWebACL` (acronym-cased) via S1 threading the namespace registry — \
         constructing + matching that exact variant round-trips 7; got {got:?}"
    );
}
