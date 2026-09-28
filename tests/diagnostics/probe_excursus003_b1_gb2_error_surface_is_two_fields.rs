//! Excursus 003 strike B1, GB2 — `:wat::core::Error`'s registered surface has EXACTLY
//! `message` and `location`. Drive it against the real `TypeEnv` the stdlib freeze
//! builds, not a text-grep of `wat/core.wat` — a registration bug (the derive silently
//! keeping a retired field, or a stale cached surface) would not show up in the source
//! text at all.
//!
//! `AUDIT-the-shape-of-an-error.md` F3, RULING 2026-09-27 item 1: `causes` leaves the
//! floor. Before this strike the surface was `{message location causes}`.
//!
//! Mutation (recorded in the strike report, not re-encoded here): re-add
//! `causes <- (:wat::core::Vector :- [:wat::core::Error])` to the `defsurface` in
//! `wat/core.wat`, alone (leaving `:wat::core::Fault` at two fields) — driven: RED, but
//! by a CASCADE, not this test's own `assert_eq!`: the trivial fixture no longer freezes
//! at all, because `:wat::core::Fault` (still `{message location}`) stops structurally
//! satisfying the now three-field `Error` surface, and `wat/spawn.wat`'s own
//! `:wat::kernel::Failure` construction (which passes a `Fault` where `[e <- Error]` is
//! declared) refuses at check time with a `TypeMismatch` naming exactly that. The
//! `.expect(...)` on `startup_beside` fails first. This is still the gate doing its job:
//! any surface-shape drift away from `{message location}` reddens it, even before this
//! test's own field-count assertion gets a chance to run.

use wat::freeze::startup_beside;
use wat::types::{SurfaceMember, TypeDef};

#[test]
fn error_surface_is_exactly_message_and_location() {
    // `TypeEnv::with_builtins()` registers only the Rust-builtin + `wat_record_from!`-derived
    // types; `:wat::core::Error` is a `defsurface` declared IN `wat/core.wat` itself and only
    // reaches the registry through a real stdlib freeze — drive one, don't fake the registry.
    let world = startup_beside(file!()).expect("the trivial fixture must freeze cleanly");
    let types = &world.types;
    let def = types
        .get(":wat::core::Error")
        .unwrap_or_else(|| panic!(":wat::core::Error must be a registered type"));
    let surface = match def {
        TypeDef::Surface(s) => s,
        other => panic!(":wat::core::Error must be a defsurface; got {other:?}"),
    };

    let field_names: Vec<&str> = surface
        .members
        .iter()
        .map(|m| match m {
            SurfaceMember::Field { name, .. } => name.as_str(),
            SurfaceMember::Method { name, .. } => {
                panic!(":wat::core::Error must have no method members; got method `{name}`")
            }
        })
        .collect();

    assert_eq!(
        field_names,
        vec!["message", "location"],
        ":wat::core::Error's surface must be exactly {{message location}} (excursus 003 \
         strike B1 removed `causes`, F3) — got {field_names:?}"
    );
}
