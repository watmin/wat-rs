//! Arc 278 BRIEF-client-validates-locally — the `RequestTooLarge`/`RequestMalformed` ctors are
//! built from the op's DECLARED response type (`build_op_response_type_constants`,
//! `src/types.rs`), never guessed by `<OpPascal>Response` string concatenation.
//!
//! `:probe::Odd::Verdict` (the response type for op `put`) is deliberately NOT named
//! `PutResponse` — the acceptance shape, mirroring `wat-scripts/scratch-pad/probe-repl-durable-
//! forms.wat`'s `EvalResponse`: a still-guessing call site fails on it first.
//!
//! PROMOTED from a throwaway used during the deliberate-break verification: nothing else in the
//! corpus exercises `op-methods`' generated function by its own SERVICE-namespaced name (every
//! fixture calls the surface name, which resolves through the Path-B runtime intrinsic in
//! `src/runtime.rs` instead), and the one existing per-op-guard fixture
//! (`probe_arc278_per_op_enforcement_codegen.wat`) uses a conventionally-named response type, so
//! it cannot tell a correct read from a guess that happens to land on the same string.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `op-methods`' OWN generated client method (`:probe::oddsvc/put`, the SERVICE name — not the
/// surface name, which would route through Path B and prove nothing about this generator)
/// refuses an over-budget request locally, returning `RequestTooLarge` built off the DECLARED
/// `Verdict` response type.
#[test]
fn op_methods_over_budget_refused_locally() {
    let got = call_beside_value(file!(), ":user::op-methods-over-budget-refused-locally")
        .unwrap_or_else(|e| panic!("op-methods' own generated method must refuse locally; got raise: {e:?}"));
    assert!(
        matches!(got, Value::i64(n) if n > 100),
        "expected RequestTooLarge.bytes > 100 (the declared cap); got {got:?}"
    );
}

/// `serve-op-arms`' per-op SIZE guard (server-side codegen) fires for a request sized between
/// `:max-request-bytes` (100) and `:max-frame-bytes`/FOO (65536) — proving the codegen guard,
/// not the transport-level FOO check, is what catches it — and builds `RequestTooLarge` off the
/// DECLARED `Verdict` response type.
#[test]
fn serve_op_arms_size_guard_fires() {
    let got = call_beside_value(file!(), ":user::serve-op-arms-size-guard-fires")
        .unwrap_or_else(|e| panic!("serve-op-arms' per-op size guard must fire; got raise: {e:?}"));
    assert!(
        matches!(got, Value::i64(n) if n > 100),
        "expected RequestTooLarge.bytes > 100 (the declared cap); got {got:?}"
    );
}

/// `serve-op-arms`' shape guard (`:wat::edn::validate`) fires for a right-tag, wrong-shape
/// request (an `i64` field carrying a String) and builds `RequestMalformed` off the DECLARED
/// `Verdict` response type — EXACT DATA, not a loose contains: the path names the offending
/// field, the declared/actual EDN shapes are named precisely.
#[test]
fn serve_op_arms_shape_guard_fires() {
    let got = call_beside_value(file!(), ":user::serve-op-arms-shape-guard-fires")
        .unwrap_or_else(|e| panic!("serve-op-arms' shape guard must fire; got raise: {e:?}"));
    match got {
        Value::String(s) => {
            assert_eq!(
                s.as_str(),
                "[\"count\"]/:wat::core::i64/String",
                "RequestMalformed.path/expected/got did not name the offending `count` field"
            );
        }
        other => panic!("expected a String, got {other:?}"),
    }
}
