//! RED probe — arc 293: a surface's structural members are introduced by the `:features` clause
//! (builder-crowned word, 2026-06-29; one canonical path — the bare member-vector form retires).
//!
//! `(defsurface :S :features [members])` and `(defsurface :S :nature :<kw> :features [members])` are
//! the only shapes; a member vector NOT preceded by `:features` is a malformed declaration.
//!
//! RED at HEAD: `parse_defsurface` (surface.rs:292) accepts only arity 2 (`[members]`) / 4
//! (`:nature X [members]`); the `:features` keyword makes the form arity 3 / 5 → `MalformedDecl`
//! ("got N args after head") → the world fails to start. GREEN once `:features` introduces the members.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// A surface declared with `:features` parses; a record satisfies it; the accessor dispatches.
#[test]
fn features_clause_introduces_surface_members() {
    match call_beside(file!(), ":geo::demo") {
        Ok(Value::String(s)) if &*s == "red" => {}
        other => panic!("expected \"red\" via the :features surface accessor; got {:?}", other),
    }
}
