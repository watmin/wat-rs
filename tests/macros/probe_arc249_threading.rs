//! FM-2-bis probe for Arc 249 — threading macros `:wat::core::->` (thread-first) +
//! `:wat::core::->>` (thread-last).
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN): plain fn-first `(map f xs)` — no threading.
//!   - MINT (GREEN): all 5 threading mints pass via FQDN registered-macro dispatch.
//!   - WITNESS: empty-list-step failure shapes (arc 249 perimeter closure).
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_threading

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — plain fn-first map, NO threading. GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_fn_first_map_no_threading() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_regression.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT — threading.
// ═══════════════════════════════════════════════════════════════════════════

/// `(:wat::core::->> [1 2 3] (map INC))` → [2 3 4]. Collection lands LAST.
#[test]
fn mint_thread_last_single_step() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_tl_single.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::->> [1 2 3] (map INC) (filter GT2))` → [3 4].
#[test]
fn mint_thread_last_pipeline() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_tl_pipeline.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::-> 5 (i64::- 3))` → 2. Accumulator injected FIRST.
#[test]
fn mint_thread_first_injects_first() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_tf_first.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

/// `(:wat::core::->> 5 (i64::- 3))` → -2. Injected LAST.
#[test]
fn mint_thread_last_injects_last() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_tl_last.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

/// Bare-keyword step: `(:wat::core::-> 3 :my::inc)` → 4.
#[test]
fn mint_bare_symbol_step() {
    let world = startup_from_file("tests/macros/probe_arc249_threading_bare_sym.wat").expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .expect("eval");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// ITEM 3 WITNESSES — empty-list-step failure shapes
// ═══════════════════════════════════════════════════════════════════════════

/// `(-> x ())` — empty list step raises at macro-expansion time.
#[test]
fn witness_thread_first_empty_step_panics_at_expansion() {
    fn attempt() -> Result<(), String> {
        startup_from_file("tests/macros/probe_arc249_threading_witness_tf_empty.wat")
            .map(|_| ())
            .map_err(|e| format!("startup: {:?}", e))
    }
    let result = std::panic::catch_unwind(attempt);
    match result {
        Err(payload) => {
            let ap = payload
                .downcast::<wat::assertion::AssertionPayload>()
                .expect("panic payload should be AssertionPayload");
            assert!(
                ap.message.contains("head") || ap.message.contains("first"),
                "unexpected panic message: {}",
                ap.message
            );
        }
        Ok(Ok(())) => panic!("expected failure but startup succeeded"),
        Ok(Err(e)) => {
            assert!(
                e.contains("first") || e.contains("no child") || e.contains("no element"),
                "unexpected error from empty -> step: {}",
                e
            );
        }
    }
}

/// `(->> x ())` — empty step desugars at expansion to `(acc)` i.e. `(5)`.
/// Startup succeeds; eval fails with MalformedForm.
#[test]
fn witness_thread_last_empty_step_desugars_to_call_on_acc() {
    fn attempt_startup() -> Result<wat::freeze::FrozenWorld, String> {
        startup_from_file("tests/macros/probe_arc249_threading_witness_tl_empty.wat")
            .map_err(|e| format!("startup: {:?}", e))
    }
    let world = std::panic::catch_unwind(attempt_startup)
        .expect("startup must not panic for ->> empty step")
        .expect("startup must succeed for ->> empty step");

    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    let eval_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &env)
    }));
    match eval_result {
        Ok(Err(e)) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("MalformedForm") || msg.contains("call head"),
                "expected MalformedForm eval error, got: {}",
                msg
            );
        }
        Ok(Ok(_)) => panic!("expected MalformedForm eval error but eval succeeded"),
        Err(_) => panic!("expected MalformedForm eval error but eval panicked"),
    }
}
