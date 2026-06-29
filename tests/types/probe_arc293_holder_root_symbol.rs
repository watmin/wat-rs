//! RED probe — arc 293 item-2a: a surface's `:holder` bound takes the holder-root SYMBOL
//! (`:wat::core::Record`), not the magic shorthand `:record`. A 0-member `:holder` surface is "any
//! aggregate of that holder" — the portability shape behind `program::Env`'s `user.program`
//! ("must be at minimum a record").
//!
//! RED at HEAD: `parse_defsurface` (surface.rs:322) hand-matches `:struct`/`:record`/`:holon-record`,
//! so `:holder :wat::core::Record` is a `MalformedDecl` → the world won't start. GREEN once `:holder`
//! routes through `Holder::from_root_keyword` (the holder-root symbol, magic shorthand annihilated).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

/// A surface declared with the holder-root SYMBOL parses, and a record satisfies its bound.
#[test]
fn surface_holder_root_symbol_accepts_record() {
    let world = startup_beside(file!()).expect("startup: a `:holder :wat::core::Record` surface must parse");
    let ast = wat::parse_one!("(:env::feed)").expect("parse feed");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::i64(42)) => {}
        other => panic!("expected i64(42) from a record satisfying the portable surface; got {:?}", other),
    }
}

/// A struct does NOT satisfy a `:holder :wat::core::Record` surface — the holder bound is a hard reject.
#[test]
fn surface_holder_root_symbol_rejects_struct() {
    let r = startup_from_file("tests/types/probe_arc293_holder_root_symbol_bad.wat");
    assert!(
        r.is_err(),
        "a struct must not satisfy a `:holder :wat::core::Record` surface (non-portable, Struct < Record); got Ok"
    );
}
