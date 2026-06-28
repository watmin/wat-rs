//! Arc 284 — disconfirming probe: there is no `:wat::core::string::interpolate` (RED at HEAD).
//!
//! `interpolate` is the PURE-TOTAL string-interpolation INTRINSIC (intueri-named): same `{name}` +
//! trailing `:name val` kwargs grammar as the `format` macro (and the same `{{`/`}}` escape + unquoted
//! render), but a Rust intrinsic that interpolates at CALL time. Because it is pure-total it is
//! expand-time-legal — usable INSIDE defmacro bodies, where the `format` macro is refused (arc 249 F5).
//! That is its whole reason to exist (the arc-277 sweep's macro-body concats need a legal format target).
//!
//! At HEAD `:wat::core::string::interpolate` is undefined → RED. GREEN when arc 284 ships the intrinsic.
//!
//! Run: cargo test --release -p wat --test probe_arc284_interpolate -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_bare, startup_beside};
use wat::runtime::{Environment, Value};

// Runtime interpolation: named slots, unquoted render (String as itself, i64 as digits), {{ }} escape.
#[test]
fn interpolate_runtime_named_unquoted_escaped() {
    let world = startup_bare().expect("startup");
    let ast = wat::parse_one!(
        r#"(:wat::core::string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5)"#
    ).expect("parse interpolate call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("interpolate undefined at HEAD: {e:?}")).value_owned();
    match got {
        Value::String(ref s) => assert_eq!(s.as_str(), "x::5 {lit}",
            "named + unquoted (string/i64) + {{{{ }}}} escape; got {s:?}"),
        other => panic!("interpolate must return String; got {other:?}"),
    }
}

// THE LOAD-BEARING PROPERTY: interpolate is legal at EXPAND time (inside a defmacro body), unlike the
// format macro. A macro that builds a keyword name via interpolate at expand time must expand cleanly.
// Fixture: tests/resolve/probe_arc284_interpolate.wat

#[test]
fn interpolate_is_legal_at_expand_time() {
    let world = startup_beside(file!())
        .expect("a defmacro body using string::interpolate must expand cleanly (the whole point)");
    let ast = wat::parse_one!("(:user::probe)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new()).expect("probe eval").value_owned();
    match got {
        Value::String(ref s) => assert_eq!(s.as_str(), "hello::built", "expand-time interpolate; got {s:?}"),
        other => panic!("expected String; got {other:?}"),
    }
}
