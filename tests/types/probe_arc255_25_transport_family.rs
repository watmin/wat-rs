//! Stone 255.25 (C-b4) — the markers are a `Transport` (arc 255).
//!
//! WHY: `:wat::kernel::Shared` / `:wat::kernel::Wire` were two EMPTY `defstruct`s while their
//! own comment said "type arguments only, not values". A struct reads IMPURE, so a marker alone
//! was refused wherever purity is asked (a `Record` field; the process child's self-peer wall —
//! 57 process-child tests red in 255.17a when the child main spelled its transport `Wire`).
//! They are now the variants of ONE closed `Pure` family (`wat/spawn.wat`):
//! `(:wat::core::defenum :wat::kernel::Transport :wat::enum::Pure :Shared [] :Wire [])`.
//!
//! Rows (the purity rows the brief fixed before the strike):
//! - `address_shared_field` — `(Address :- [S R Transport.Shared])` is an in-process resource:
//!   IMPURE, refused as a pure `Record` field (`is_pure_type`'s `Address` arm, keyed on
//!   `is_shared_marker`). Pre-stone, the same row spelled `:wat::kernel::Shared` was refused
//!   the same way — the property is kept across the respelling.
//! - `address_wire_field` — the twin on `Transport.Wire`: pure, accepted.
//! - `markers_are_pure` — `Transport.Shared`, `Transport.Wire`, and `Transport` itself as
//!   fields: accepted. Pre-stone, `:wat::kernel::Shared` as a field was REFUSED
//!   (`ImpureFieldInPureAggregate`, "impure (struct) type").
//! - `impure_variant_field` — the CONTROL: a unit variant of an `Impure` family, same position,
//!   is refused. A variant's purity is read from its enum's DECLARATION, not from the
//!   "unknown path ⇒ type parameter ⇒ pure" arm.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

const DIR: &str = "tests/types/probe_arc255_25_transport_family";

fn accepted(suffix: &str) {
    let path = format!("{DIR}_{suffix}.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}

/// The one refusal every negative row expects: `(aggregate, field, field_ty)`.
fn refused_impure_field(suffix: &str) -> (String, String, String) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    match startup_from_file(&path) {
        Ok(_) => panic!("{path} must be refused (ImpureFieldInPureAggregate); it froze clean"),
        Err(StartupError::Type(e)) => match e.into_kind() {
            TypeErrorKind::ImpureFieldInPureAggregate { aggregate, field, field_ty } => {
                (aggregate, field, field_ty)
            }
            other => panic!("{path}: expected ImpureFieldInPureAggregate, got {other:?}"),
        },
        Err(other) => panic!("{path}: expected StartupError::Type, got {other:?}"),
    }
}

#[test]
fn an_address_on_the_shared_transport_is_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("address_shared_field");
    assert_eq!(aggregate, ":probe::HoldsShared");
    assert_eq!(field, "addr");
    // rune:lint(no-inlined-wat) — the `(:wat::kernel::Address :- [...])` literal below is the
    // golden rendered TYPE NAME the checker prints, compared by equality; not wat source that is
    // evaluated.
    assert_eq!(
        field_ty,
        "(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])"
    );
}

#[test]
fn an_address_on_the_wire_transport_is_pure() {
    accepted("address_wire_field");
}

#[test]
fn the_transport_markers_alone_are_pure() {
    accepted("markers_are_pure");
}

#[test]
fn a_variant_of_an_impure_family_is_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("impure_variant_field");
    assert_eq!(aggregate, ":probe::HoldsLease");
    assert_eq!(field, "lease");
    assert_eq!(field_ty, ":probe::Lease.Held");
}
