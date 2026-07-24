//! Arc 278 peer-lifecycle walls — the `poll'`/`select'` ServiceEvent swallow-gate (strike 1).
//!
//! WHY: `poll'`/`select'` already return a matchable `:wat::spawn::ServiceEvent` (Message/Closed/
//! Lost[cause]/Malformed[cause]/Rejected[cause]) — value-faced, a peer failure is a variant, never a
//! raise. But nothing forced FACING it: `(:wat::core::let [_ (:wat::kernel::select' peers)] …)` (or a
//! do-non-final drop) silently dropped the event, hiding a `Lost`/`Malformed` failure. `ServiceEvent`
//! is now must-use (added to `MUST_USE_PARAMETRIC_HEADS`), so a dropped event is a located compile
//! error. A *faced* event has an arm-joined type, not `ServiceEvent`, so this fires ONLY on a raw drop.
//! This completes the swallow-axis for the reactor multiplexers, matching recv'/send'.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn discarded_service_event_in_let_underscore_is_compile_error() {
    let err = startup_from_file("tests/comms/probe_arc278_service_event_must_use_wall.wat.bad")
        .expect_err("a discarded ServiceEvent bound to `_` in a `let` must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::let"
            && reason.contains(":wat::spawn::ServiceEvent")
            && reason.contains("must be faced"));
}
