//! RED probe — arc 293 inheritance annihilation: a recordtype/aggregatetype parent MUST be
//! a holder-root. A USER-type parent (nominal inheritance) is REJECTED at registration.
//!
//! The model (`AGGREGATE-MODEL.md` §4): a type is `holder + own fields`, flat — there is no
//! `:parent`; the only thing a parent slot may hold is a holder-root (= the holder). Reuse-of-shape
//! is surface-splice (`[~@:Surface own <- :T]`), never a nominal base.
//!
//! RED at HEAD: `register_with_span` (types.rs:457) registers `:my::Child <: :my::Base` for ANY
//! existing parent → startup SUCCEEDS, so `is_err()` is false and this asserts-fail. GREEN once the
//! holder-root guard rejects a non-holder-root parent at registration.

use wat::freeze::startup_from_file;

/// A `recordtype` whose parent is a USER type (inheritance) must be rejected at registration.
#[test]
fn recordtype_with_user_parent_is_rejected() {
    let r = startup_from_file("tests/types/probe_arc293_reject_user_parent_bad.wat");
    assert!(
        r.is_err(),
        "a recordtype with a USER-type parent (inheritance) must be rejected — \
         the parent must be a holder-root; got Ok"
    );
}
