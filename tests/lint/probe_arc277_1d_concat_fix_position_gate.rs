//! Arc 277.1d — the concat fix picks the right head by POSITION (defmacro-body → interpolate intrinsic,
//! defn-body → format macro).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc277_1d_concat_fix_position_gate.wat`,
//! slurped via `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust
//! string, never `format!`-assembled). The fixture's `:t::fix` lints+fixes a SourceFile carrying BOTH a
//! defmacro-body concat (expand-time → must become `interpolate`) and a defn-body concat (runtime → must
//! stay `format`); the probe eval_in_frozen's `(:t::fix)` and asserts each head was picked by position.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1d_concat_fix_position_gate

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn concat_fix_picks_head_by_position() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::fix)").expect("parse the fix call");
    let fixed = match eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-fix-file raised: {e:?}"))
        .value_owned()
    {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected String; got {other:?}"),
    };
    assert_eq!(
        fixed,
        concat!(
            "(:wat::core::defmacro :u::m [x <- :wat::WatAST] -> :wat::core::String ",
            "(:wat::core::let [s (:wat::core::ast-name x) nm ",
            "(:wat::core::string::interpolate ",
            "\"{s}::Op\" :s s)] nm)) ",
            "(:wat::core::defn :u::f [a <- :wat::core::String] -> :wat::core::String ",
            "(:wat::core::format ",
            "\"x: {a}\" :a a))"
        ),
        "defmacro-body → interpolate, defn-body → format; must match position-gate golden"
    );
}
