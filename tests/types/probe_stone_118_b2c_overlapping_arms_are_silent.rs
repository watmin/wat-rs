//! Stone 118.B2c strike 1 — WITNESS: two `defclause` arms with the IDENTICAL declared type and
//! DIFFERENT bodies are accepted silently; the second body is dead code.
//!
//! Found 2026-08-18 while ruling door 1's fix shape, from the builder's question: *"this is the same
//! concept as how re-defs work? you may only express something's def once and all other attempts
//! must be identical."* Checking that turned up a live defect rather than an analogy.
//!
//! ## Why this is a hole in the redef rule, not merely an oddity
//!
//! Arc 054 made `typealias` / `define` / `defmacro` *"if byte-equivalent, no-op"*, else
//! `DuplicateDefine`; arc 157 added the opt-in `redef_allowed` with a type-stability check.
//! **`defclause` arms were never covered** — an arm is not a definition BY NAME — so the one
//! registry that dispatches on TYPES is the one registry with no define-once rule. Dispatch is
//! first-match-wins in declaration order (`src/runtime.rs`: the clause loop returns on first match,
//! no most-specific selection), so the later body is unreachable and nothing says so.
//!
//! ## ⚠ THIS FILE IS A WITNESS AND IT INVERTS
//!
//! `defect_*` asserts the BROKEN behaviour, so it is green on the broken substrate. **When strike 1
//! lands, it must go RED** — the fixture then fails to REGISTER, and this test is replaced by one
//! asserting the named refusal. Kept rather than described in prose
//! (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`); not `#[ignore]`d
//! (`[[feedback_a_house_convention_can_be_the_mechanism_that_built_the_pile]]`).
//!
//! ## ★ The control is load-bearing
//!
//! `control_disjoint_arms_still_dispatch` proves a normal multi-arm `defclause` works. Without it,
//! "overlapping arms are refused" would be satisfied by a substrate that refused ALL multi-arm
//! defclauses — which would take the whole language with it.

use wat::freeze::call_beside_value;

fn call_string(entry: &str) -> String {
    let v = call_beside_value(file!(), entry)
        .unwrap_or_else(|e| panic!("{entry} must evaluate; got {e:?}"));
    match v {
        wat::Value::String(s) => s.to_string(),
        other => panic!("{entry}: expected a String, got {other:?}"),
    }
}

#[test]
fn defect_duplicate_arm_is_accepted_and_the_second_body_is_dead() {
    assert_eq!(
        call_string(":my::which"),
        "FIRST",
        "two arms with IDENTICAL declared types are accepted silently and the FIRST wins — the \
         second body is unreachable. When 118.B2c strike 1 arms the overlap wall this fixture must \
         fail to REGISTER, and this test inverts into an assertion on the named refusal."
    );
}

#[test]
fn control_disjoint_arms_still_dispatch() {
    assert_eq!(call_string(":my::describe-int"), "an int");
    assert_eq!(call_string(":my::describe-string"), "a string");
    assert_eq!(call_string(":my::describe-bool"), "a bool");
}
