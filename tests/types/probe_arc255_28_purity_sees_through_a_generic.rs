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
//! `79febbaba` binary, before the stone):
//! - `bare_shared_field` / `bare_shared_peer` — CONTROL A: the bare Shared address as a pure
//!   `Record` field, and as a wire peer's payload at the `self-peer` producer. Refused pre and post.
//! - ⭐ `generic_shared_field` / ⭐ `generic_shared_peer` — SUBJECT B: the same address through the
//!   `Pure` generic enum, in the same two positions. Pre: ACCEPTED. Now refused.
//! - `generic_wire_field` / `generic_wire_peer` — the `Transport.Wire` twins: accepted.
//! - `generic_struct_field` — a generic `Struct` over `i64`: impure (pre: accepted).
//! - `generic_impure_enum_field` — a generic `Impure` enum over `i64`: impure (pre: accepted).
//! - `unbound_var` — a generic record whose field is `(E :- [T])`, `T` unbound: accepted; purity
//!   is decided at instantiation. Its twin `unbound_var_instantiated_shared` (two generic layers,
//!   over Shared) is refused (pre: accepted).
//! - `self_referential` — a self-referential generic over Wire: terminates, accepted. Its twin
//!   `self_referential_shared` is refused (pre: accepted) — the recursion guard swallows nothing.
//
// rune:lint(no-inlined-wat) — the `(:probe::… :- [...])` literals below are golden rendered TYPE
// NAMES the checker prints, compared by equality; not wat source that is evaluated.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

const DIR: &str = "tests/types/probe_arc255_28_purity_sees_through_a_generic";

const SHARED_ADDRESS: &str =
    "(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])";

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

/// The §7 peer wall at the `self-peer` producer: exactly one refusal, naming `payload`.
fn refused_impure_peer(suffix: &str, payload: &str) {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    assert_eq!(errs.len(), 1, "{path}: {errs:?}");
    let expected = format!(
        "a wire peer (Peer<I,O>) carries only pure data — type {payload} is not \
         pure (§7 purity wall). If this peer is used only within a thread \
         (in-locus, shared memory), use ThreadSelfPeer<I,O> — any I/O types \
         are allowed in-locus. If this peer must cross a process boundary \
         (wire), redesign I/O types to use records, scalars, or pure enums \
         (no Sender/Receiver/handle fields)."
    );
    match &errs[0].kind {
        CheckErrorKind::MalformedForm { head, reason, .. } => {
            assert_eq!(head, ":wat::program::self-peer");
            assert_eq!(reason, &expected);
        }
        other => panic!("{path}: expected MalformedForm from self-peer, got {other:?}"),
    }
}

#[test]
fn control_a_the_bare_shared_address_is_an_impure_field() {
    let (aggregate, field, field_ty) = refused_impure_field("bare_shared_field");
    assert_eq!(aggregate, ":probe::HoldsBare");
    assert_eq!(field, "addr");
    assert_eq!(field_ty, SHARED_ADDRESS);
}

#[test]
fn control_a_the_bare_shared_address_is_an_impure_peer_payload() {
    refused_impure_peer("bare_shared_peer", SHARED_ADDRESS);
}

#[test]
fn a_shared_address_through_a_pure_generic_enum_is_an_impure_field() {
    let (aggregate, field, field_ty) = refused_impure_field("generic_shared_field");
    assert_eq!(aggregate, ":probe::HoldsGeneric");
    assert_eq!(field, "status");
    assert_eq!(field_ty, "(:probe::E :- [:wat::kernel::Transport.Shared])");
}

#[test]
fn a_shared_address_through_a_pure_generic_enum_is_an_impure_peer_payload() {
    refused_impure_peer("generic_shared_peer", "(:probe::E :- [:wat::kernel::Transport.Shared])");
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
fn the_unbound_generic_instantiated_over_shared_is_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("unbound_var_instantiated_shared");
    assert_eq!(aggregate, ":probe::HoldsCarrier");
    assert_eq!(field, "carrier");
    assert_eq!(field_ty, "(:probe::Carrier :- [:wat::kernel::Transport.Shared])");
}

#[test]
fn a_self_referential_generic_terminates() {
    accepted("self_referential");
}

#[test]
fn a_self_referential_generic_over_shared_is_impure() {
    let (aggregate, field, field_ty) = refused_impure_field("self_referential_shared");
    assert_eq!(aggregate, ":probe::HoldsChain");
    assert_eq!(field, "chain");
    assert_eq!(field_ty, "(:probe::Chain :- [:wat::kernel::Transport.Shared])");
}
