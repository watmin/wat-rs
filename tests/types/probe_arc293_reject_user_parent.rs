//! RED probe — arc 293 inheritance annihilation: a recordtype/aggregatetype parent MUST be
//! a nature-root. A USER-type parent (nominal inheritance) is REJECTED at registration.
//!
//! The model (`AGGREGATE-MODEL.md` §4): a type is `nature + own fields`, flat — there is no
//! `:parent`; the only thing a parent slot may hold is a nature-root (= the nature). Reuse-of-shape
//! is surface-splice (`[~@:Surface own <- :T]`), never a nominal base.
//!
//! RED at HEAD: `register_with_span` (types.rs:457) registers `:my::Child <: :my::Base` for ANY
//! existing parent → startup SUCCEEDS, so `is_err()` is false and this asserts-fail. GREEN once the
//! nature-root guard rejects a non-nature-root parent at registration.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

/// A `recordtype` whose parent is a USER type (inheritance) must be rejected at registration.
#[test]
fn recordtype_with_user_parent_is_rejected() {
    let r = startup_from_file("tests/types/probe_arc293_reject_user_parent.wat.bad");
    wat::assert_startup_error!(r,
        StartupError::Type(e) if matches!(e.kind(), TypeErrorKind::MalformedDecl { head, reason }
            if head == "recordtype"
            && reason == "parent ':my::Base' is not a nature-root; inheritance is unsupported — \
                           reuse a shape via surface-splice `[~@:Surface …]`")
    );
}
