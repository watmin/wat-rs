//! Arc 293.4d-fix — `parse_defsurface` must NOT silently drop members written outside the `[...]` vector.
//!
//! The silent-swallow (found at 293.4d): a `defsurface` with 4 args after the head where arg[1] is the member
//! vector (not `:nature`) — i.e. method members written as separate top-level args (the stale `definterface` shape)
//! — had its trailing members SILENTLY DROPPED. A surface could be declared weaker than written, with no error, so
//! satisfaction would pass types that don't actually expose the dropped members. extirpare: the parser must reject it.
//!
//! At HEAD (pre-fix) `startup_from_file` SUCCEEDS (the methods are silently dropped). After the fix it ERRORS.
//!
//! Phase 3 (296-L) repair: arc 278 S4c made `:nature` MANDATORY on 2026-07-07 (38f31069f), which
//! postdates this probe (2026-06-28, 35ba08636). Without `:nature`/`:features`, the fixture died
//! at the mandatory-`:nature` arity gate (`MalformedDecl "expected \`:nature :<kw>\` after the
//! surface name"`) before the parser ever reached the member-vector logic this probe exists to
//! pin — so the bare `is_err()` was passing for an unrelated reason. The fixture now carries
//! `:nature :wat::core::Struct :features [...]` so the leftover-args-after-the-member-vector
//! shape is reachable again, and the assertion below names the ACTUAL error it raises.

use wat::freeze::{startup_from_file, StartupError};
use wat::types::TypeErrorKind;

/// A surface with method members written OUTSIDE the member vector must be rejected, not silently truncated.
#[test]
fn members_outside_the_member_vector_are_rejected_not_silently_dropped() {
    let result = startup_from_file("tests/types/probe_arc293_4d_fix_silent_member_drop.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Type(e) if matches!(e.kind(), TypeErrorKind::MalformedDecl { head, reason }
            if head == ":wat::core::defsurface"
            && reason == "unexpected form after the member vector — every surface member (a field \
                           `name <- :T` AND a method `(name [self] -> :ret)`) goes INSIDE the single \
                           `[...]` member vector; nothing follows it")
    );
}
