//! Arc 293.4a — method members in `defsurface` (the disconfirming probe).
//!
//! THE GAP this isolates: a `defsurface` whose member list contains a METHOD member
//! `(area [self] -> :f64)` (a list, not a `[name <- :T]` field triple) must (1) PARSE
//! and (2) be SATISFIED by a type that backs the method with a `defn :T/name`.
//!
//! RED at HEAD — `parse_defsurface` (src/types/surface.rs:48) runs the member vector
//! through `argspec::parse_argspec_triples` (field triples only); the method member
//! does not parse, so the program fails to type-check.
//!
//! GREEN at 293.4a — `SurfaceDef.members` carries Field|Method members (the arc-232
//! method-sig parser, `parse_defprotocol_form` / `ProtocolMethodSig`, lifted), and
//! `struct_satisfies_surface` treats a Method member as satisfied by a matching
//! `defn :T/name`. NO dispatcher (`:Shape/area s`) is exercised here — that is 293.4b.

use wat::freeze::startup_beside;

/// A `defsurface` mixing a field member + a method member must parse, and a record
/// that backs the method with a `defn :T/area` must structurally satisfy it.
#[test]
#[ignore = "RED at HEAD: arc-293.4a (method members in defsurface — parse + satisfy) not built; \
            un-ignore when the strike lands GREEN"]
fn method_member_surface_parses_and_is_satisfied_by_a_defn() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "a defsurface with a method member must parse, and a record backing it with a \
         `defn :T/area` must satisfy it structurally; got: {:?}",
        world.err()
    );
}
