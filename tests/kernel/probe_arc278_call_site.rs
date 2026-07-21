//! Arc 278 "caller.1" — `(:wat::kernel::call-site)` native nullary verb probe.
//!
//! Verifies the verb returns the CALLER's `:wat::kernel::Frame` (file/line/symbol),
//! mirroring the mechanism `:wat::kernel::assertion-failed!` uses (src/assertion.rs) —
//! `snapshot_call_stack().first()` from inside a native verb IS the caller's frame
//! (native verbs push no `FrameGuard` of their own; only wat fn-calls do).
//!
//! RED at HEAD: `:wat::kernel::call-site` is unknown to the type checker → startup
//! fails (unresolved-verb error).
//!
//! GREEN after: startup succeeds; the deftest' fn RETURNS (not raises) — the returned
//! Frame's file/line/symbol are all `Some` and describe the caller.
//!
//! WAT fixture: tests/kernel/probe_arc278_call_site.wat

use wat::freeze::startup_from_file;
use wat::runtime::apply_function;

fn run_test_fn(path: &str, name: &str) -> Result<(), String> {
    let world = startup_from_file(path)
        .expect("startup should succeed (:wat::kernel::call-site must exist + type-check)");
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("no {name} in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{e:?}")),
    }
}

/// `(:wat::kernel::call-site)` returns the caller's Frame — a passing deftest' RETURNS
/// cleanly (not raises) when the returned file/line/symbol describe the caller (this
/// fixture file, a positive line, and the "probe::here" symbol).
#[test]
fn call_site_returns_caller_frame() {
    let r = run_test_fn(
        "tests/kernel/probe_arc278_call_site.wat",
        ":user::call-site-returns-caller-frame",
    );
    assert!(r.is_ok(), "call-site's returned Frame must describe the caller; got Err: {r:?}");
}
