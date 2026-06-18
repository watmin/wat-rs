//! Arc 277.1d — disconfirming probe: the concat→format fix ignores POSITION (RED at HEAD).
//!
//! 277.1c-fix rewrites a bare-symbol concat to `(:wat::core::format …)` ALWAYS — including inside a
//! defmacro body, where `format` (a macro) is refused at expand time (arc 249 F5). The arc-277 sweep
//! broke the whole stdlib for exactly this. arc 284 shipped `:wat::core::string::interpolate` (pure-total,
//! expand-time-legal). This gate makes the fix POSITION-AWARE: a concat in a defmacro body → the
//! `interpolate` INTRINSIC; a concat in a defn body → the `format` MACRO (zero-cost). One template shape,
//! head chosen by position.
//!
//! At HEAD: a defmacro-body bare-symbol concat is rewritten to `format` (illegal there) → no
//! `string::interpolate` in the output → RED. GREEN when the gate ships.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1d_concat_fix_position_gate -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A source with BOTH: a defmacro whose body builds a name via a bare-symbol concat (expand-time →
// must become interpolate), and a defn whose body has a bare-symbol concat (runtime → must stay format).
const SRC: &str = r#"(:wat::core::defmacro :u::m [x <- :wat::WatAST] -> :wat::core::String (:wat::core::let [s (:wat::core::ast-name x) nm (:wat::core::string::concat s \"::Op\")] nm)) (:wat::core::defn :u::f [a <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a))"#;

#[test]
#[ignore = "arc 277.1d — RED until the concat-fix position gate ships; un-ignore on green"]
fn concat_fix_picks_head_by_position() {
    let prog = format!("(:wat::lint::lint-fix-file (:wat::source::File \"t.wat\" \"{SRC}\"))");
    let world = startup_from_source("(:wat::core::defn :user::main [] -> :wat::core::nil nil)", None,
        Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(&prog).expect("parse lint-fix-file");
    let fixed = match eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}")).value_owned() {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected String; got {other:?}"),
    };
    // The defmacro-body concat → interpolate INTRINSIC (expand-time-legal).
    assert!(fixed.contains("(:wat::core::string::interpolate \"{s}::Op\" :s s)"),
        "defmacro-body concat must become interpolate; got: {fixed}");
    // The defn-body concat → format MACRO (zero-cost runtime).
    assert!(fixed.contains("(:wat::core::format \"x: {a}\" :a a)"),
        "defn-body concat must stay format; got: {fixed}");
    assert!(!fixed.contains("string::concat"), "both concats must be rewritten; got: {fixed}");
}
