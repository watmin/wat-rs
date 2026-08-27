//! RED probe — arc 293 item-2a: a surface's `:nature` bound takes the nature-root SYMBOL
//! (`:wat::core::Record`), not the magic shorthand `:record`. A 0-member `:nature` surface is "any
//! aggregate of that nature" — the portability shape behind `program::Env`'s `user-data`
//! ("must be at minimum a record").
//!
//! RED at HEAD: `parse_defsurface` (surface.rs:322) hand-matches `:struct`/`:record`/`:holon-record`,
//! so `:nature :wat::core::Record` is a `MalformedDecl` → the world won't start. GREEN once `:nature`
//! routes through `Nature::from_root_keyword` (the nature-root symbol, magic shorthand annihilated).

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

/// A surface declared with the nature-root SYMBOL parses, and a record satisfies its bound.
#[test]
fn surface_nature_root_symbol_accepts_record() {
    match call_beside_value(file!(), ":env::feed") {
        Ok(Value::i64(42)) => {}
        other => panic!("expected i64(42) from a record satisfying the portable surface; got {:?}", other),
    }
}

/// A struct does NOT satisfy a `:nature :wat::core::Record` surface — the nature bound is a hard reject.
#[test]
fn surface_nature_root_symbol_rejects_struct() {
    let r = startup_from_file("tests/types/probe_arc293_holder_root_symbol.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":env::take"
            && param == "#1"
            && expected == ":env::Portable"
            && got == ":env::Stru"
    );
}
