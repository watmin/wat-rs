//! FM-2-bis probe for Arc 249 — threading macros `:wat::core::->` (thread-first) +
//! `:wat::core::->>` (thread-last).
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN): plain fn-first `(map f xs)` — no threading.
//!   - MINT (GREEN): all 5 threading mints pass via FQDN registered-macro dispatch.
//!   - WITNESS: empty-list-step failure shapes (arc 249 perimeter closure).
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_threading

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside_value` because this probe drives several distinct co-located fixtures from one `.rs`.)
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval")
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — plain fn-first map, NO threading. GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_fn_first_map_no_threading() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_regression.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT — threading.
// ═══════════════════════════════════════════════════════════════════════════

/// `(:wat::core::->> [1 2 3] (map INC))` → [2 3 4]. Collection lands LAST.
#[test]
fn mint_thread_last_single_step() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_tl_single.wat");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::->> [1 2 3] (map INC) (filter GT2))` → [3 4].
#[test]
fn mint_thread_last_pipeline() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_tl_pipeline.wat");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::-> 5 (i64::- 3))` → 2. Accumulator injected FIRST.
#[test]
fn mint_thread_first_injects_first() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_tf_first.wat");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::->> 5 (i64::- 3))` → -2. Injected LAST.
#[test]
fn mint_thread_last_injects_last() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_tl_last.wat");
    assert_eq!(result, Value::bool(true));
}

/// Bare-keyword step: `(:wat::core::-> 3 :my::inc)` → 4.
#[test]
fn mint_bare_symbol_step() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_bare_sym.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// ITEM 3 WITNESSES — empty-list-step failure shapes
// ═══════════════════════════════════════════════════════════════════════════

/// `(-> x ())` — empty list step raises at macro-expansion time.
#[test]
fn witness_thread_first_empty_step_panics_at_expansion() {
    fn attempt() -> Result<(), wat::freeze::StartupError> {
        startup_from_file("tests/macros/probe_arc249_threading_witness_tf_empty.wat").map(|_| ())
    }
    let result = std::panic::catch_unwind(attempt);
    match result {
        Err(_payload) => {
        }
        Ok(Ok(())) => panic!("expected failure but startup succeeded"),
        Ok(Err(e)) => {
            wat::assert_edn_matches_file!(
                format!("{:?}", e),
                "probe_arc249_threading__witness_thread_first_empty_step_panics_at_expansion.edn",
                "empty -> step must match macro-expansion failure golden"
            );
        }
    }
}

/// `(->> x ())` — empty step desugars at expansion to `(acc)` i.e. `(5)`.
/// Startup succeeds; eval fails with MalformedForm.
#[test]
fn witness_thread_last_empty_step_desugars_to_call_on_acc() {
    fn attempt_startup() -> Result<wat::freeze::FrozenWorld, wat::freeze::StartupError> {
        startup_from_file("tests/macros/probe_arc249_threading_witness_tl_empty.wat")
    }
    let world = std::panic::catch_unwind(attempt_startup)
        .expect("startup must not panic for ->> empty step")
        .expect("startup must succeed for ->> empty step");

    let func = world
        .symbols()
        .get(":user::compute")
        .expect("no :user::compute in fixture")
        .clone();
    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
    }));
    match eval_result {
        Ok(Err(e)) => {
            let msg = format!("{:?}", e);
            wat::assert_edn_matches_file!(
                msg,
                "probe_arc249_threading__witness_thread_last_empty_step_desugars_to_call_on_acc.edn",
                "empty ->> step must match MalformedForm eval golden"
            );
        }
        Ok(Ok(_)) => panic!("expected MalformedForm eval error but eval succeeded"),
        Err(_) => panic!("expected MalformedForm eval error but eval panicked"),
    }
}
