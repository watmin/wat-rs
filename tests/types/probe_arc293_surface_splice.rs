//! Arc 293 — surface-splice in aggregate field vectors (the disconfirming probe).
//!
//! THE GAP this isolates: a `defrecord`/`defstruct`/`defholon` field vector may reuse a
//! surface's ATTRIBUTES via `~@:Surface` — inlining them flat before the own fields
//! (AGGREGATE-MODEL principle 4; DESIGN.md:130 "spliceable into bodies for DRY"). Designed
//! in arc 293, never built: `parse_aggregate_fields` (src/types/defstruct.rs:297) runs the
//! vector through `argspec::parse_argspec_triples` (field triples only), so the reader's
//! `(:wat::core::unquote-splicing :Surface)` node (parser.rs:353) trips
//! "name must be a plain symbol". Zero `~@` usage in the whole corpus — it rotted unbuilt.
//!
//! GREEN when: splice-expansion (at the type-registration pass, where surfaces are already
//! registered — surfaces-before-records load-order) resolves each `~@:Surface` to the
//! surface's `SurfaceMember::Field` members and inlines them; the merged field list is the
//! union (spliced-then-own, first-occurrence order); a name repeated at an IDENTICAL type
//! dedupes, a name repeated at a CONFLICTING type is a MalformedDecl.
//!
//! ⛔ IGNORE-LEDGER(293-surface-splice): un-ignore `surface_splice_merges_and_constructs`
//! as the FINAL green step of the build; it must pass. (examinare: the RED gate.)

use wat::freeze::{startup_beside, startup_from_file, StartupError};
use wat::types::TypeErrorKind;

/// POSITIVE (the RED gate): a `defrecord` splicing two surfaces' attributes + an own field
/// must parse, construct positionally over the merged field list, and expose a `:Rec/field`
/// accessor for each spliced field. RED at HEAD (splice unbuilt); GREEN when the build ships.
#[test]
fn surface_splice_merges_and_constructs() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "a defrecord splicing `~@:Scope ~@:Named value` must parse, construct, and expose \
         accessors for the spliced fields; got: {:?}",
        world.err()
    );
}

/// NEGATIVE: two splices installing the same field name at CONFLICTING types
/// (`foobar` :i64 vs :String) must be REJECTED (the merge's type-consistency rule).
/// Not a clean gate at HEAD (splice unbuilt → also errors), so it is the build's own
/// correctness check, verified once the positive is green.
#[test]
fn surface_splice_conflicting_field_types_rejected() {
    let world = startup_from_file("tests/types/probe_arc293_surface_splice.wat.bad");
    wat::assert_startup_error!(world,
        StartupError::Type(e) if matches!(e.kind(), TypeErrorKind::MalformedDecl { head, reason }
            if head == "recordtype"
            && reason == "surface-splice conflict: field `foobar` is installed at conflicting \
                           types (Path(\":wat::core::i64\") vs Path(\":wat::core::String\")) by \
                           two splices (or a splice and an own field) — a field repeated across \
                           splices must carry an identical type")
    );
}
