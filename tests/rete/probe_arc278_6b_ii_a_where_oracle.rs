//! Arc 278 stone 6b-ii-a — `where`/TestNode in the oracle (`compile-condition` + `fire-rules$oracle`) + the compile fence.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! Probed through `fire-rules$oracle` — 6b-ii-a builds the oracle TestNode; the native kernel port +
//! differential are 6b-ii-b. A `where` is a left-only filter: it keeps a token iff `eval-test` of its
//! expr against the token's bindings is true. The compile fence rejects a `where` whose expr is not
//! (pure ∧ deterministic ∧ total ∧ primitive?). Live mouths: `compile-all`, `fire-rules$oracle`,
//! `query`, `eval-test`.
//!
//! Run: cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle

use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_file, FrozenWorld, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

// Paths to the co-located .wat fixtures (relative to the crate root).
const WORLD_CMP_PATH: &str    = "tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat";
const WORLD_USERFN_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat";
const WORLD_IMPURE_PATH: &str = "tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat";

/// Call the named zero-arg entry fn in `world_path`'s fixture and return its count.
/// The rete compile fence rejects an impure/non-deterministic condition by PANICKING
/// (Option/expect → panic_any — the engine's compile-rejection mechanism, same as raise!).
/// Catch it so a rejection surfaces as Err, not an uncaught test panic. (Before the arc-296
/// None-fix an illegal `(:wat::core::None)` form threw a *catchable* UnknownFunction here
/// instead — that form was never legal and is now corrected; the fence's real reject is a panic.)
fn run_count(world_path: &str, fn_name: &str) -> Result<Value, StartupError> {
    let world: FrozenWorld = startup_from_file(world_path)?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        // arc 296 Stone L: preserve the fence's `AssertionPayload.message` instead of a generic
        // sentinel — the sentinel is exactly what made the corresponding `.is_err()` assertion
        // vacuous (mirrors `probe_arc278_then_user_forms.rs`'s `run`, the sibling probe this
        // module's own doc comment names). Arc 296 Stone M: the fields land in the REAL
        // RuntimeErrorKind::AssertionFailed shape (mirrors AssertionPayload's own layout)
        // instead of being flattened to a bare String.
        Err(panic_payload) => {
            let (message, actual, expected) = match panic_payload.downcast_ref::<AssertionPayload>() {
                Some(p) => (p.message.clone(), p.actual.clone(), p.expected.clone()),
                None => {
                    let message = panic_payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                        .unwrap_or_else(|| "panic-opaque".to_string());
                    (message, None, None)
                }
            };
            Err(StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::AssertionFailed { message, actual, expected },
            ))))
        }
        Ok(res) => res.map_err(|e| StartupError::Runtime(Box::new(e))),
    }
}

/// 1 — the where PASSES: Temp(5), (> 5 0) true → exactly one Gate derived.
#[test]
fn where_passes_when_predicate_true() {
    let r = run_count(WORLD_CMP_PATH, ":user::run-gate-c5");
    assert!(matches!(r, Ok(Value::i64(1))), "where (> 5 0) true → 1 Gate; got {r:?}");
}

/// 2 — the where BLOCKS: Temp(-5), (> -5 0) false → zero Gates (the filter actually filters).
#[test]
fn where_blocks_when_predicate_false() {
    let r = run_count(WORLD_CMP_PATH, ":user::run-gate-cneg5");
    assert!(matches!(r, Ok(Value::i64(0))), "where (> -5 0) false → 0 Gates; got {r:?}");
}

/// 3 — a USER-fn predicate in the where works through the network: big?(150) → one Gate.
#[test]
fn where_with_user_fn_predicate_passes() {
    let r = run_count(WORLD_USERFN_PATH, ":user::run-gate-c150");
    assert!(matches!(r, Ok(Value::i64(1))), "where (big? 150) true → 1 Gate; got {r:?}");
}

/// 3b — the same user-fn predicate blocks below threshold: big?(50) → zero.
#[test]
fn where_with_user_fn_predicate_blocks() {
    let r = run_count(WORLD_USERFN_PATH, ":user::run-gate-c50");
    assert!(matches!(r, Ok(Value::i64(0))), "where (big? 50) false → 0 Gates; got {r:?}");
}

/// 4 — the compile FENCE rejects an impure `where` (io): compiling the rule raises.
#[test]
fn fence_rejects_impure_where_at_compile() {
    // Grounded via `./target/release/wat` on a scratch `:user::main` invoking the same body:
    // the compile fence's `AssertionPayload.message`, now preserved by `run_count` above
    // instead of collapsed to a generic sentinel.
    let r = run_count(WORLD_IMPURE_PATH, ":user::run-gate-c5");
    wat::assert_startup_error!(
        r,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::AssertionFailed { message, .. }
                if message == "compile-condition: where expr is not pure — ':wat::io::IOReader/open-file' is not pure"
        )
    );
}
