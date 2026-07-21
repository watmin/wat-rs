//! arc 278 §4 — the `:wat::telemetry::log` call-site widget captures each `(log …)` call's OWN
//! source line (per-log-line `emitted-from`), end-to-end. See DESIGN-telemetry-caller-and-capacity.md §4.
//!
//! Two adjacent `(log …)` calls inside one `with-span`, written through the span to a MemStore-backed
//! journal, then queried back: their `emitted-from` lines must differ by EXACTLY 1 — proving the widget
//! bakes `(:wat::kernel::macro-call-site)` at each call's own line (a constant frame would give 0). The
//! span stamps the correlation `uuid` so the Logs join the metrics; the widget only supplies emitted-from.
//!
//! RED at HEAD: `:wat::telemetry::log` does not exist → unknown callee → startup fails.
//! GREEN after: `:user::log-line-diff` returns `1`.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn log_captures_call_line() {
    let world = startup_beside(file!())
        .expect("startup should succeed (:wat::telemetry::log widget + span'/journal'/mem-store' baked)");
    let func = world
        .symbols()
        .get(":user::log-line-diff")
        .unwrap_or_else(|| panic!(":user::log-line-diff not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("the `log` widget e2e raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(1)),
        "two adjacent (log …) calls must capture lines differing by exactly 1 (got {got:?}; \
         -1 = count != 2 / same-nanos collision, -2 = query-logs not Success)"
    );
}
