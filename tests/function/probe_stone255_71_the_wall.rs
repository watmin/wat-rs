//! Arc 255 STONE 71 (THE WALL) — every collection constructor call requires its own `:- [T…]`
//! type bracket, whatever the head's spelling, for all seven heads (Vector, PersistentVector,
//! HashMap, PersistentMap, HashSet, List, Tuple). See the co-located `.wat.bad`'s header.
//!
//! `Vector`/`HashMap`/`HashSet` already enforced this before this stone (arc 109 stone 3); their
//! rows here are a REGRESSION guard, not new behavior. `List`/`PersistentMap`/`PersistentVector`/
//! `Tuple`'s rows are the wall THIS stone raises.
//!
//! Quirk carried over unchanged (out of this stone's scope — `Vector`'s bracket-less error was
//! already wired before this stone): `infer_list_constructor`'s own bracket-less `MalformedForm`
//! hardcodes `head: ":wat::core::vec"` (the RETIRED verb spelling) regardless of whether the call
//! site actually reads `Vector` or the old `vec` name — code wins over what a reader might expect
//! ("Vector" in the error), so this test asserts what the checker actually emits.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const FIXTURE: &str = "tests/function/probe_stone255_71_the_wall.wat.bad";

fn errors() -> Vec<wat::check::error::CheckError> {
    let err = startup_from_file(FIXTURE)
        .expect_err("every bracket-less constructor call in this fixture must fail check");
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("expected a type-check error");
    };
    errs
}

fn count_with_head(errs: &[wat::check::error::CheckError], head: &str) -> usize {
    errs.iter()
        .filter(|e| matches!(&e.kind, CheckErrorKind::MalformedForm { head: h, .. } if h == head))
        .count()
}

#[test]
fn exactly_fourteen_errors_one_per_bracketless_call() {
    // 7 heads x 2 spellings = 14 bracket-less calls, each its own located MalformedForm.
    assert_eq!(errors().len(), 14, "expected exactly 14 check errors, one per (head, spelling) row");
}

#[test]
fn vector_both_spellings_refused() {
    // Regression guard (arc 109's pre-existing wall) — NOT new this stone. `head` is the
    // retired `:wat::core::vec` spelling (see module doc); both keyword and symbol calls route
    // through the same `infer_list_constructor`, so both trip this same head string.
    assert_eq!(count_with_head(&errors(), ":wat::core::vec"), 2);
}

#[test]
fn hashmap_both_spellings_refused() {
    // Regression guard (arc 109's pre-existing wall) — NOT new this stone.
    assert_eq!(count_with_head(&errors(), ":wat::core::HashMap"), 2);
}

#[test]
fn hashset_both_spellings_refused() {
    // Regression guard (arc 109's pre-existing wall) — NOT new this stone.
    assert_eq!(count_with_head(&errors(), ":wat::core::HashSet"), 2);
}

#[test]
fn persistentvector_both_spellings_refused() {
    let errs = errors();
    assert_eq!(count_with_head(&errs, ":wat::core::PersistentVector"), 2);
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::PersistentVector"
            && reason.contains("PersistentVector")
            && reason.contains("wat.type/PersistentVector :- [T"));
}

#[test]
fn persistentmap_both_spellings_refused() {
    let errs = errors();
    assert_eq!(count_with_head(&errs, ":wat::core::PersistentMap"), 2);
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::PersistentMap"
            && reason.contains("PersistentMap")
            && reason.contains("wat.type/PersistentMap :- [T"));
}

#[test]
fn list_both_spellings_refused() {
    let errs = errors();
    assert_eq!(count_with_head(&errs, ":wat::core::List"), 2);
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::List"
            && reason.contains("List")
            && reason.contains("wat.type/List :- [T"));
}

#[test]
fn tuple_both_spellings_refused() {
    let errs = errors();
    assert_eq!(count_with_head(&errs, ":wat::core::Tuple"), 2);
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::Tuple"
            && reason.contains("Tuple")
            && reason.contains("wat.type/Tuple :- [T"));
}
