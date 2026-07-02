//! Integration coverage for arc 143 slice 1 — three substrate
//! introspection primitives:
//!   `:wat::runtime::lookup-define`
//!   `:wat::runtime::signature-of-defn`
//!   `:wat::runtime::body-of`
//!
//! Each primitive takes a keyword name and returns
//! `:Option<wat::holon::HolonAST>`. Test coverage:
//!   1. User-define lookup — define a wat function, call the primitive,
//!      assert the returned Option is Some.
//!   2. Substrate-primitive lookup — call the primitive on
//!      `:wat::core::foldl`, assert Some.
//!   3. Unknown name — call on a non-existent name, assert None.
//!   4. `body-of` for substrate primitive returns None (not the sentinel).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn unwrap_bool(v: Value, ctx: &str) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("{}: expected bool; got {:?}", ctx, other),
    }
}

fn unwrap_string(v: Value, ctx: &str) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("{}: expected String; got {:?}", ctx, other),
    }
}

// ─── :wat::runtime::lookup-define ───────────────────────────────────────────

#[test]
fn lookup_define_user_define_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test1-lookup-user)"), "lookup-user"),
        "lookup-define user function should return Some"
    );
}

#[test]
fn lookup_define_substrate_primitive_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test2-lookup-foldl)"), "lookup-foldl"),
        ":wat::core::foldl is a substrate primitive; lookup-define must return Some"
    );
}

#[test]
fn lookup_define_unknown_name_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::test3-lookup-none)"), "lookup-none"),
        "unknown name should return None (fn returns true for None)"
    );
}

// ─── :wat::runtime::signature-of-defn ───────────────────────────────────────

#[test]
fn signature_of_defn_user_define_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test4-sig-user)"), "sig-user"),
        "user-defined function signature-of-defn should return Some"
    );
}

#[test]
fn signature_of_defn_substrate_primitive_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test5-sig-foldl)"), "sig-foldl"),
        ":wat::core::foldl synthesised head must be Some"
    );
}

#[test]
fn signature_of_defn_unknown_name_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::test6-sig-none)"), "sig-none"),
        "unknown name should return None"
    );
}

// ─── :wat::runtime::body-of ─────────────────────────────────────────────────

#[test]
fn body_of_user_define_returns_some() {
    assert!(
        unwrap_bool(run_expr("(:t::test7-body-user)"), "body-user"),
        "user-defined function body-of should return Some"
    );
}

#[test]
fn body_of_substrate_primitive_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::test8-body-prim-none)"), "body-prim-none"),
        "substrate primitives have no wat body — body-of must return None"
    );
}

#[test]
fn body_of_unknown_name_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::test9-body-unknown-none)"), "body-unknown-none"),
        "unknown name should return None"
    );
}

// ─── Shape verification via edn::write ───────────────────────────────────

#[test]
fn signature_of_defn_foldl_renders_synthesised_shape() {
    let line = unwrap_string(run_expr("(:t::test10-sig-render)"), "sig-render");
    assert_eq!(
        line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::foldl<T_Acc> #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "_a0" #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :Fn #wat-edn.holon/Keyword :Acc #wat-edn.holon/Keyword :T #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :Acc]] #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "_a1" #wat-edn.holon/Keyword :Acc] #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "_a2" #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::Vector #wat-edn.holon/Keyword :T]] #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :Acc]"#,
        "foldl signature must render with synthesised param names and type params"
    );
}

#[test]
fn lookup_define_user_function_contains_defn_keyword() {
    let line = unwrap_string(run_expr("(:t::test11-def-render)"), "def-render");
    assert_eq!(
        line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defn #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :t::my-square #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "x" #wat-edn.holon/Keyword :wat::core::i64] #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :wat::core::i64] #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::* #wat-edn.holon/Symbol "x" #wat-edn.holon/Symbol "x"]]"#,
        "rendered define-ast must show defn head and my-square name"
    );
}
