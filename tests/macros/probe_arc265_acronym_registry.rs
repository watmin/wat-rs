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

use std::sync::Arc;

use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): the fixture's fwd/rev/roundtrip/rev-default fns take a String arg (no
// zero-arg entry to call_beside_value), so fetch the fn from the sibling fixture and apply_function it
// directly with a Rust-constructed Value::String arg — no inline wat driver expression.
fn call_string(world: &wat::freeze::FrozenWorld, fn_name: &str, arg: &str) -> String {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    let arg = Value::String(Arc::new(arg.to_string()));
    match apply_function(func, vec![arg], world.symbols(), wat::rust_caller_span!()).expect("eval") {
        Value::String(v) => (*v).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn namespace_scoped_acronym_conversion_restores_casing() {
    let world = startup_beside(file!()).expect("startup");
    // my::aws has ACL declared → forward emits the acronym as one segment, reverse restores it.
    assert_eq!(call_string(&world, ":user::fwd", "CreateWebACL"), "create-web-acl");
    assert_eq!(call_string(&world, ":user::rev", "create-web-acl"), "CreateWebACL");
    assert_eq!(call_string(&world, ":user::roundtrip", "CreateWebACL"), "CreateWebACL");
    // :other::ns has NO acronyms → default: ACL is not restored.
    assert_eq!(call_string(&world, ":user::rev-default", "create-web-acl"), "CreateWebAcl");
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
    let func = world
        .symbols()
        .get(":user::req-n")
        .expect("no :user::req-n in fixture")
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("req-n raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected 7: the surface `:my::aws::Waf` (ACL declared BEFORE it registers) must synthesize \
         `:my::aws::Waf::Op::CreateWebACL` (acronym-cased) via S1 threading the namespace registry — \
         constructing + matching that exact variant round-trips 7; got {got:?}"
    );
}
