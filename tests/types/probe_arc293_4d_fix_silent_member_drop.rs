//! Arc 293.4d-fix — `parse_defsurface` must NOT silently drop members written outside the `[...]` vector.
//!
//! The silent-swallow (found at 293.4d): a `defsurface` with 4 args after the head where arg[1] is the member
//! vector (not `:holder`) — i.e. method members written as separate top-level args (the stale `definterface` shape)
//! — had its trailing members SILENTLY DROPPED. A surface could be declared weaker than written, with no error, so
//! satisfaction would pass types that don't actually expose the dropped members. extirpare: the parser must reject it.
//!
//! At HEAD (pre-fix) `startup_from_file` SUCCEEDS (the methods are silently dropped). After the fix it ERRORS.

use wat::freeze::startup_from_file;

/// A surface with method members written OUTSIDE the member vector must be rejected, not silently truncated.
#[test]
fn members_outside_the_member_vector_are_rejected_not_silently_dropped() {
    let result = startup_from_file("tests/types/probe_arc293_4d_fix_silent_member_drop_bad.wat");
    assert!(
        result.is_err(),
        "a defsurface with members written outside the `[...]` vector must be a hard error \
         (the members were being silently dropped); but startup succeeded"
    );
}
