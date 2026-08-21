//! Arc 278 — BRIEF-construction-inside-a-fn.md: unblocks Stone B's headline (`ac90d262` shipped
//! the RHS fence mechanism and said plainly a `defn` that CONSTRUCTS and returns a record was
//! still refused, because its macro-expanded body bottoms out in `:wat::core::kwargs-construct` /
//! `:wat::core::aggregate-new`, both unclassified in `purity.rs`'s `intrinsic_meta`).
//!
//! Three worlds, mirroring `probe_arc278_then_user_forms.rs`'s own co-located-fixture +
//! `startup_from_file` pattern:
//!
//!   - `probe_construction_headline_green.wat` — GREEN, the headline itself: a `defn` that
//!     CONSTRUCTS and returns a fresh record from bound `:then` terms (not merely extracts an
//!     existing one, unlike the prior stone's workaround fixture). Fired through BOTH the oracle
//!     (`fire-rules$oracle`) and the native kernel (`fire-rules`), same expected value.
//!   - `probe_construction_headline_red.wat` — RED: the item's head fn BOTH constructs a record
//!     AND touches a genuinely impure op. Classifying the two construction verbs pure must not
//!     open a hole — the compile fence must still refuse it, naming the impure head.
//!   - `probe_construction_arity_check_rejects.wat.bad` — the checker-gap closure (gap (a) in
//!     `intrinsic_meta`'s classification comment): a direct wrong-arity
//!     `:wat::core::aggregate-new` call must now be a `--check`-time (i.e. `startup_from_file`-
//!     time) rejection with a located `ArityMismatch`, not a runtime-only surprise.
//!
//! Run: cargo test --release -p wat --test probe_construction_headline

use wat::assertion::AssertionPayload;
use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

const WORLD_GREEN: &str = "tests/rete/probe_construction_headline_green.wat";
const WORLD_RED: &str = "tests/rete/probe_construction_headline_red.wat";
const WORLD_ARITY_BAD: &str = "tests/rete/probe_construction_arity_check_rejects.wat.bad";

/// Call the named zero-arg entry fn and return its result, or an `Err` string for either an
/// ordinary raise OR the fence's `Option/expect` panic (caught via `catch_unwind`, exactly as
/// `probe_arc278_then_user_forms.rs`'s `run` does).
fn run(world_path: &str, fn_name: &str) -> Result<Value, String> {
    let world = startup_from_file(world_path).map_err(|e| format!("startup: {e:?}"))?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| format!("eval: {e:?}")),
        Err(panic_payload) => {
            if let Some(p) = panic_payload.downcast_ref::<AssertionPayload>() {
                Err(p.message.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

/// GREEN, oracle path: a `defn` that CONSTRUCTS `(:cg::Rate :count 7 :window 9)` from literal
/// `:then` args and returns it — unconfounded: no `cg::Rate` fact of count=7 exists before the
/// rule fires; only the construction produces one.
#[test]
fn construct_and_return_derives_via_oracle() {
    let r = run(WORLD_GREEN, ":user::run-oracle");
    assert!(matches!(r, Ok(Value::i64(7))), "expected count=7 via the oracle; got {r:?}");
}

/// GREEN, NATIVE path: same rule, same expected value, through the compiled RHS path
/// (`fire-rules`/`insert`) instead of the interpreted oracle — proves compiled == interpreted.
#[test]
fn construct_and_return_derives_via_native_kernel() {
    let r = run(WORLD_GREEN, ":user::run-native");
    assert!(matches!(r, Ok(Value::i64(7))), "expected count=7 via the native kernel; got {r:?}");
}

/// RED: a fn that BOTH constructs a record AND touches an impure op is still refused, naming the
/// impure head — classifying `aggregate-new`/`kwargs-construct` pure did not open a hole.
#[test]
fn construct_plus_impure_op_still_refused() {
    let r = run(WORLD_RED, ":user::run-compile");
    let msg = r.expect_err("a :then item head that touches an impure op must fail to compile, even if it also constructs a record");
    assert_eq!(
        msg,
        "compile-condition: then expr is not pure — ':wat::io::IOReader/open-file' is not pure"
    );
}

/// Gap (a) closure: a direct, wrong-arity `:wat::core::aggregate-new` call is a `--check`-time
/// (startup) rejection, not a runtime-only surprise. A `.wat.bad` fixture is EXPECTED to fail to
/// load — `startup_from_file` itself must return `Err`, and the located error must name the
/// callee and the arity mismatch.
#[test]
fn direct_aggregate_new_bad_arity_is_a_check_time_rejection() {
    let err = startup_from_file(WORLD_ARITY_BAD)
        .expect_err("a wrong-arity direct aggregate-new call must fail to start up (checker rejection)");
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — the `Debug` rendering embeds an absolute file path (Span),
    // non-deterministic across machines/CI (same reason `probe_arc278_then_user_forms.rs`'s
    // `non_fact_return_type_is_refused` uses `contains`, not `assert_eq!`) — assert the
    // load-bearing SUBSTANCE (error kind, offending callee, actual/expected arity), not the
    // whole rendered blob.
    assert!(msg.contains("ArityMismatch"), "must be an ArityMismatch, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains(":wat::core::aggregate-new"), "must name the offending callee, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("expected 2") && msg.contains("got 1"), "must name the actual/expected arity, got:\n{msg}");
}
