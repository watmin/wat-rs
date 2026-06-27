//! Arc 293.4a — DISCONFIRMING PROBE: a `defsurface` with a METHOD member, satisfied by a record's
//! `:T/method` defn.
//!
//! **The wat source is the co-located sibling fixture** `probe_arc293_4a_surface_method_member.wat`,
//! slurped via `startup_beside(file!())` — the repo's test-fixture scheme: a probe's fixture inherits
//! the probe's name and lives at its side (never inlined as a Rust string; never named in `demos/`,
//! which is for curated showpieces). A `.wat` fixture is `cargo wat`-runnable + fix-wat-migratable.
//!
//! Contract (`DESIGN-293.4-strike` § 4a): a surface member may be a METHOD accessor `(name [self …] -> ret)`,
//! not only a field `[name <- :T]`; a type satisfies it by exposing a matching `defn :T/name`.
//!
//! RED at HEAD: `parse_defsurface` (`src/types/surface.rs`) is FIELD-ONLY → the method member is malformed
//! → load errors. GREEN at 293.4a: the method member parses + `:geo::Box` satisfies `:geo::Sized`.

use wat::freeze::startup_beside;

#[test]
#[ignore = "RED at HEAD: arc-293.4a (method members in defsurface) not built; un-ignore when it lands — \
            the strike's disconfirming probe, kept #[ignore]'d so the floor=0 gate stays green"]
fn defsurface_method_member_satisfied_by_record_defn() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "a defsurface METHOD member must parse and a record exposing :T/size must satisfy it; got: {:?}",
        world.err()
    );
}
