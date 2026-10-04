//! Excursus 003 strike F, GF2 — no record named `::Fault`/`::Failure` fails the surface.
//!
//! AUDIT-the-shape-of-an-error.md F8 / BRIEF-shape-strike-F: a record named `Fault`/
//! `Failure` claims to be an error. This standing gate walks the FULL registered stdlib
//! TypeEnv (via `startup_bare` — the real freeze, never a text-grep of the `.wat` sources,
//! which would miss a registration bug the same way GB2's precedent does for `:wat::core::
//! Error`'s own shape) and asserts: every registered Aggregate whose name ends in
//! `::Fault` or `::Failure` either structurally satisfies `:wat::core::Error`
//! ({message location}, by field name+type — the surface has exactly these two leaf
//! fields, so a name+type field check IS the structural check here) or IS
//! `:wat::kernel::Failure` (the envelope — `{error frames frames-elided}`, which
//! deliberately does NOT itself satisfy Error: it is the thing that CARRIES one).
//!
//! Before this strike this gate is RED at anchors `:wat::cache::Fault`,
//! `:wat::sqlite::Fault`, `:wat::query::Fault` (no `location`) and `:wat::doctest::Failure`
//! (no `location`, and never will — it is a domain outcome, which is why it was RENAMED
//! to `:wat::doctest::Violation` rather than conformed: the rename is what makes it stop
//! matching this gate's `::Failure` suffix at all).
//!
//! Mutation (recorded, not re-encoded here): re-add a non-conforming `Fault` — e.g. revert
//! any one of the three conforming records' `location` field — RED, this gate's own
//! `assert!` naming that type.

use wat::freeze::startup_bare;
use wat::types::{TypeDef, TypeExpr};

/// `fields` carries `("message", String)` structurally equal to `:wat::core::Error`'s own
/// two leaf features — the surface has no methods and no other fields (locked by
/// probe_excursus003_b1_gb2_error_surface_is_two_fields.rs), so a name+type match on these
/// two IS the structural satisfaction check for this closed, two-leaf-field surface.
fn satisfies_error_surface(fields: &[(String, TypeExpr)]) -> bool {
    let has = |name: &str, ty: &str| {
        fields
            .iter()
            .any(|(n, t)| n == name && matches!(t, TypeExpr::Path(p) if p == ty))
    };
    has("message", ":wat::core::String") && has("location", ":wat::core::Span")
}

#[test]
fn every_fault_or_failure_named_type_conforms_or_is_the_envelope() {
    let world = startup_bare().expect("the bare stdlib must freeze cleanly");
    let types = world.types();

    let mut violations: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (name, def) in types.iter() {
        if !(name.ends_with("::Fault") || name.ends_with("::Failure")) {
            continue;
        }
        if name == ":wat::kernel::Failure" {
            // The envelope — exempt by name, per the brief's own carve-out. It does NOT
            // itself satisfy Error (it carries one), which this loop never checks.
            continue;
        }
        checked += 1;
        match def {
            TypeDef::Aggregate(a) => {
                if !satisfies_error_surface(&a.fields) {
                    let field_names: Vec<&str> = a.field_names().collect();
                    violations.push(format!("{name} (fields: {field_names:?})"));
                }
            }
            other => violations.push(format!("{name} (not an Aggregate: {other:?})")),
        }
    }

    assert!(
        checked >= 3,
        "sanity: expected to find at least the three conforming AUDIT F8 names \
         (:wat::cache::Fault, :wat::sqlite::Fault, :wat::query::Fault — the fourth, \
         :wat::doctest::Failure, was RENAMED to :wat::doctest::Violation and so no longer \
         carries this suffix at all); found only {checked} — the registry walk itself may \
         be broken"
    );
    assert!(
        violations.is_empty(),
        "GF2: {} record(s) named ::Fault/::Failure fail :wat::core::Error \
         and are not the envelope: {violations:?}",
        violations.len()
    );
}
