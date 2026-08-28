//! GREEN probe — arc 293 K3-revise: the PAIR of projection verbs (`to-record` / holon `to-record`).
//!
//! A surface emits the PAIR of backing records (`$core-record` + `$holon-record`).
//! Projection is ONE-WAY UP — never down to a struct (you already have the struct in locus;
//! `$struct` is the impure tier and no longer exists as a surface backing type).
//!
//! GREEN after K3-revise: `to-record` + `holon::to-record` are bound; the PAIR is emitted;
//! `to-struct` is GONE (unbound; dispatch arm removed from runtime.rs and check.rs).

use wat::freeze::{eval_in_frozen, startup_beside, call_beside_value};
use wat::runtime::{Environment, RuntimeErrorKind, Value};

#[test]
fn pair_projection_emits_core_and_holon_records() {
    match call_beside_value(file!(), ":k3::demo") {
        Ok(Value::i64(7)) => {}
        other => panic!("expected 7 (y=4 from $core-record + x=3 from $holon-record); got {other:?}"),
    }
}

#[test]
fn to_struct_is_gone_at_runtime() {
    // After K3-revise the `:wat::core::to-struct` dispatch arm is deleted.
    // A call to it must fail (UnknownFunction / unbound verb) — never silently succeed.
    let world = startup_beside(file!())
        .expect("startup must succeed (fixture does not use to-struct)");
    // The call form lives in a co-located expression FRAGMENT (not the `.wat` fixture, and not an
    // inlined Rust string) — read + parsed here at runtime, deliberately bypassing the type-checker,
    // so this exercises the RUNTIME dispatch table specifically (see the fragment's own header comment).
    let expr_path = "tests/types/probe_arc293_k3_to_record_to_struct_call.wat.expr";
    let src = std::fs::read_to_string(expr_path)
        .unwrap_or_else(|e| panic!("expr fragment {expr_path:?} must exist: {e}"));
    let ast = wat::parse_one_with_file(&src, expr_path).expect("parse to-struct call fragment");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    // Grounded via an equivalent dynamic-dispatch route (`:wat::eval-edn!` on the same call
    // shape, since `eval_in_frozen`'s Rust API isn't independently invokable from `wat --check`):
    // the deleted verb falls through the dispatch cluster to a plain symbol lookup, which
    // fails as `RuntimeErrorKind::UnknownFunction`, not a bespoke "to-struct is gone" kind.
    assert!(
        matches!(&result, Err(e) if matches!(e.kind(), RuntimeErrorKind::UnknownFunction(path)
            if path == ":wat::core::to-struct")),
        "`:wat::core::to-struct` must be GONE (unbound); got {:?}",
        result
    );
}
