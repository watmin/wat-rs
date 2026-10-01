//! Stone 255.28 — purity sees through a generic type (arc 255).
//!
//! WHY: `is_pure_type` judged a parametric head by its TYPE ARGUMENTS only
//! (`_ => args.iter().all(is_pure_type)`); it never substituted them into the head's declared
//! fields or variants. So a Shared `Address` reached through a `Pure` generic enum
//! (`(E :- [Transport.Shared])`, whose variant field is `(Address :- [i64 i64 T])`) read PURE —
//! exactly defservice's `Status` shape (`FINDING-what-relies-on-the-thread-escape-hatch.md`). And a
//! generic `Struct` or `Impure` enum instantiated with pure arguments read pure too: its declared
//! nature was never asked. Now a declared head's purity is its declaration's purity, instantiated.
//!
//! Rows (fixtures beside this file, `probe_arc255_28_purity_sees_through_a_generic_*`; pre = the
//! `79febbaba` binary, before 255.28):
//! - `bare_shared_field` / `bare_shared_peer` — CONTROL A: the bare Shared address as a pure
//!   `Record` field, and as a wire peer's payload at the `self-peer` producer. Refused by 255.28.
//!   Inverted by 255.29: a Shared address is data, so both are accepted.
//! - ⭐ `generic_shared_field` / ⭐ `generic_shared_peer` — SUBJECT B: the same address through the
//!   `Pure` generic enum, in the same two positions. 255.28 refused them (pre-255.28: accepted).
//!   Inverted by 255.29: the substituted address is pure, so both are accepted.
//! - `generic_wire_field` / `generic_wire_peer` — the `Transport.Wire` twins: accepted.
//! - `generic_struct_field` — a generic `Struct` over `i64`: impure (pre: accepted).
//! - `generic_impure_enum_field` — a generic `Impure` enum over `i64`: impure (pre: accepted).
//! - `unbound_var` — a generic record whose field is `(E :- [T])`, `T` unbound: accepted; purity
//!   is decided at instantiation. Its twin `unbound_var_instantiated_shared` (two generic layers,
//!   over Shared) was refused by 255.28 and is accepted after 255.29.
//! - `self_referential` — a self-referential generic over Wire: terminates, accepted. Its twin
//!   `self_referential_shared` was refused by 255.28 and is accepted after 255.29 — the recursion
//!   guard still walks the chain; the address it finds is now pure.
//
// rune:lint(no-inlined-wat) — the `(:probe::… :- [...])` literals below are golden rendered TYPE
// NAMES the checker prints, compared by equality; not wat source that is evaluated.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

const DIR: &str = "tests/types/probe_arc255_28_purity_sees_through_a_generic";

fn accepted(suffix: &str) {
    let path = format!("{DIR}_{suffix}.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}

/// The record-field wall's one refusal: `(aggregate, field, field_ty)`.
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
fn control_a_the_bare_shared_address_is_a_pure_field() {
    accepted("bare_shared_field");
}

#[test]
fn control_a_the_bare_shared_address_is_a_pure_peer_payload() {
    accepted("bare_shared_peer");
}

#[test]
fn a_shared_address_through_a_pure_generic_enum_is_a_pure_field() {
    accepted("generic_shared_field");
}

#[test]
fn a_shared_address_through_a_pure_generic_enum_is_a_pure_peer_payload() {
    accepted("generic_shared_peer");
}

#[test]
fn the_wire_twin_is_a_pure_field() {
    accepted("generic_wire_field");
}

#[test]
fn the_wire_twin_is_a_pure_peer_payload() {
    accepted("generic_wire_peer");
}

#[test]
fn a_generic_struct_over_pure_args_is_still_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("generic_struct_field");
    assert_eq!(aggregate, ":probe::HoldsBox");
    assert_eq!(field, "box");
    assert_eq!(field_ty, "(:probe::Box :- [:wat::core::i64])");
}

#[test]
fn a_generic_impure_enum_over_pure_args_is_still_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("generic_impure_enum_field");
    assert_eq!(aggregate, ":probe::HoldsLease");
    assert_eq!(field, "lease");
    assert_eq!(field_ty, "(:probe::Lease :- [:wat::core::i64])");
}

#[test]
fn an_unbound_type_variable_is_decided_at_instantiation() {
    accepted("unbound_var");
}

#[test]
fn the_unbound_generic_instantiated_over_shared_is_pure() {
    accepted("unbound_var_instantiated_shared");
}

#[test]
fn a_self_referential_generic_terminates() {
    accepted("self_referential");
}

#[test]
fn a_self_referential_generic_over_shared_is_pure() {
    accepted("self_referential_shared");
}
