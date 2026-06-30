//! GREEN probe — arc 293 K3-revise: the PAIR of projection verbs (`to-record` / holon `to-record`).
//!
//! A surface emits the PAIR of backing records (`$core-record` + `$holon-record`).
//! Projection is ONE-WAY UP — never down to a struct (you already have the struct in locus;
//! `$struct` is the impure tier and no longer exists as a surface backing type).
//!
//! GREEN after K3-revise: `to-record` + `holon::to-record` are bound; the PAIR is emitted;
//! `to-struct` is GONE (unbound; dispatch arm removed from runtime.rs and check.rs).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn pair_projection_emits_core_and_holon_records() {
    let world = startup_beside(file!())
        .expect("the pair of projection verbs must emit + populate :S$core-record / :S$holon-record");
    let ast = wat::parse_one!("(:k3::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
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
    // Build the call form dynamically so the fixture's type-checker never sees it.
    let ast = wat::parse_one!("(:wat::core::to-struct (:k3::Pt 3 4) :k3::Planar)")
        .expect("parse to-struct call");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_err(),
        "`:wat::core::to-struct` must be GONE (unbound); got Ok({:?})",
        result.ok()
    );
}
