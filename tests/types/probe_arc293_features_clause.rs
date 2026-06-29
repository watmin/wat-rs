//! RED probe — arc 293: a surface's structural members are introduced by the `:features` clause
//! (builder-crowned word, 2026-06-29; one canonical path — the bare member-vector form retires).
//!
//! `(defsurface :S :features [members])` and `(defsurface :S :holder :<kw> :features [members])` are
//! the only shapes; a member vector NOT preceded by `:features` is a malformed declaration.
//!
//! RED at HEAD: `parse_defsurface` (surface.rs:292) accepts only arity 2 (`[members]`) / 4
//! (`:holder X [members]`); the `:features` keyword makes the form arity 3 / 5 → `MalformedDecl`
//! ("got N args after head") → the world fails to start. GREEN once `:features` introduces the members.

use wat::freeze::startup_beside;
use wat::runtime::{Environment, Value};
use wat::freeze::eval_in_frozen;

/// A surface declared with `:features` parses; a record satisfies it; the accessor dispatches.
#[test]
fn features_clause_introduces_surface_members() {
    let world = startup_beside(file!()).expect("startup: a `:features` surface must parse");
    let ast = wat::parse_one!("(:geo::demo)").expect("parse demo");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) if &*s == "red" => {}
        other => panic!("expected \"red\" via the :features surface accessor; got {:?}", other),
    }
}
