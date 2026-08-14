//! Arc 296 — the nature roots are the holders they NAME (the regression gate).
//!
//! ## What went wrong, and why nothing caught it
//!
//! `:wat::core::Record` and `:wat::holon::Record` are the umbrella roots of two of the three
//! holders. Both were registered `nature: Nature::Struct` — which, read against this project's
//! own AGGREGATE-MODEL trit (`Struct(−1)` impure-capable · `Record(0)` pure, crosses ·
//! `HolonRecord(+1)` pure + VSA), asserted **"a record may hold impure values"**: the exact
//! inverse of what a record is. Builder, 2026-08-15: *"this is outrageous heresy."*
//!
//! It survived because the consequences were papered over one at a time instead of at the root:
//!
//!  - `is_pure_type` grew a hardcoded `"wat::core::Record" => return true` short-circuit whose
//!    own comment named the symptom — the aggregate arm *"would return a FALSE POSITIVE impure
//!    verdict"*. A patch on the CONSUMER for a lie in the DECLARATION.
//!  - **That patch was never given to the sibling.** `:wat::holon::Record` fell through and was
//!    reported impure, so no pure aggregate could hold a field typed "any holon record" — while
//!    "any record" was fine. It was never noticed because **nothing exercised the twin**.
//!  - And it leaked out of purity into the LATTICE: `register_builtin` derives each type's
//!    subtype edge from `nature.root_keyword()`, so a Struct-natured Record umbrella emitted
//!    `:wat::core::Record <: :wat::core::Struct`, silently making every record in wat a subtype
//!    of Struct.
//!
//! Found by reading what three rows of a table SAID — not by any failing test. That is exactly
//! why this file exists: the defect's whole survival strategy was that no row asked.
//!
//! ## The four rows, and which one is load-bearing
//!
//! Rows 1–2 are the cure; row 4 is the wall that must not move. **Row 3 is the load-bearing one**
//! — it is the only row that distinguishes "fixed the cause" from "fixed the symptom." Rows 1
//! and 2 would both go green if someone simply added a second short-circuit to `is_pure_type`;
//! only row 3 flips when the DECLARATION is corrected, because only the declaration controls the
//! edge.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_from_file, StartupError};

/// The ONE assertion both red rows make: freezing `path` fails with a `TypeMismatch` where the
/// callee expected `expected` and got `got`.
///
/// Asserted STRUCTURALLY — on the `CheckErrorKind` variant and its fields — never by
/// `contains()`-ing the rendered Debug. That matters twice over here. A loose string check would
/// pass on a `TypeMismatch` about some *other* pair of types, so it could not tell "the lattice
/// refused a record from a Struct slot" from "the fixture has an unrelated typo"; and this whole
/// arc exists because a value's structure was reconstructed from its rendering instead of being
/// read. The lint caught this file doing it (third time today the wall fired on its own author).
fn assert_type_mismatch(path: &str, expected: &str, got: &str, why: &str) {
    let err = startup_from_file(path).expect_err(why);
    let StartupError::Check(errors) = err else {
        panic!("{why}\n  expected a CHECK failure; got a different StartupError: {err:?}")
    };
    let found = errors.0.iter().any(|e| {
        matches!(&e.kind, CheckErrorKind::TypeMismatch { expected: exp, got: g, .. }
            if exp == expected && g == got)
    });
    assert!(
        found,
        "{why}\n  expected a TypeMismatch of {expected:?} vs {got:?}; the check DID fail, but on \
         something else — so this fixture has stopped measuring the lattice.\n  errors: {errors:?}"
    );
}

/// Rows 1 + 2 — a pure aggregate may hold BOTH umbrellas as fields.
///
/// Row 1 (`:wat::holon::Record`) was RED before the fix. Row 2 (`:wat::core::Record`) was green
/// only because of the hand-written short-circuit; it is green here with that patch DELETED,
/// which is what makes it evidence rather than decoration.
#[test]
fn both_record_umbrellas_are_pure_enough_to_be_held_by_a_record() {
    startup_from_file("tests/types/probe_arc296_nature_roots.wat").unwrap_or_else(|e| {
        panic!(
            "a pure aggregate must be able to hold a field typed with EITHER record umbrella — \
             both are pure holders by nature. A failure naming `:wat::holon::Record` means the \
             sibling regressed to Struct-nature; one naming `:wat::core::Record` means the \
             deleted `is_pure_type` short-circuit was load-bearing after all.\n\ngot: {e:?}"
        )
    });
}

/// ⛔ ROW 3 — THE LOAD-BEARING RED. A record is NOT a struct.
///
/// The spurious `:wat::core::Record <: :wat::core::Struct` edge existed ONLY because the umbrella
/// carried the wrong nature. A green here means the edge is back — and with it the whole class,
/// because the edge and the false purity verdict have one cause.
#[test]
fn a_record_does_not_satisfy_the_struct_umbrella() {
    assert_type_mismatch(
        "tests/types/probe_arc296_nature_roots__record_is_not_a_struct.wat",
        ":wat::core::Struct",
        ":t::Pt",
        "a record must NOT be assignable to a `:wat::core::Struct` slot. If this froze clean, \
         `register_builtin` emitted `:wat::core::Record <: :wat::core::Struct` again — which \
         happens exactly when the Record umbrella is registered with a nature that is not its \
         own, so `child != root` and the edge guard stops skipping.",
    );
}

/// Row 4 — the wire wall, unmoved. A struct may hold impure values, so it can never cross a
/// comms boundary and must be refused by the record umbrella.
#[test]
fn a_struct_does_not_satisfy_the_record_umbrella() {
    assert_type_mismatch(
        "tests/types/probe_arc296_nature_roots__struct_is_not_a_record.wat",
        ":wat::core::Record",
        ":t::S",
        "a struct must NOT be assignable to a `:wat::core::Record` slot — records are the \
         wire-friendly holders and a struct may hold resources. A green here would mean the \
         nature fix WIDENED the wall rather than correcting the lattice.",
    );
}
