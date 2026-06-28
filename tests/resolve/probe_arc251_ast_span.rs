//! Forward-proof probe — Stone 251.5 / Slice 4.2a: `ast-span` (intueri-named).
//!
//! `(:wat::core::ast-span node) -> {:line N :col N}` — a plain map (HashMap<keyword,i64>) of the
//! node's source START location.
//!
//! RED at HEAD: `:wat::core::ast-span` is UnknownFunction.
//!
//! Run: `cargo test --release --test probe_arc251_ast_span`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_i64(world: &wat::freeze::FrozenWorld, call: &str) -> Result<i64, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn c01_ast_span_head_line() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_i64(&world, "(:user::c01)"), Ok(1), "head keyword line should be 1");
}

#[test]
fn c02_ast_span_head_col() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_i64(&world, "(:user::c02)"), Ok(2), "head keyword col should be 2 (just after `(`)");
}

#[test]
fn c03_ast_span_symbol_col() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_i64(&world, "(:user::c03)"), Ok(18), "symbol x col should be 18");
}
