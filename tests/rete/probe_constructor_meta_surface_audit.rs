//! BRIEF-constructor-meta-audit.md (`d6c32cf5`) audited `constructor_meta`'s two return sites
//! (`src/rete/purity.rs`) and measured `total: false` at both, with three named failure modes.
//! BRIEF-construction-total-three-walls.md (this pass) CLOSES all three — moving each fire-time
//! failure to freeze — and flips both sites to `total: true`. This file is the audit's own probe
//! module, RE-POINTED: the three ORIGINAL fixtures' assertions inverted (their SUBJECT — that
//! these forms are handled honestly — survives: the un-inverted commentary lives on each
//! fixture's own header); three new fixtures prove the "REJECT at compile" / "STILL WORKS" pair
//! for #2 and #3 (#1 has no REJECT half — it is the one wall that got WIRED, not tightened).
//!
//!   - `probe_constructor_meta_surface_pure_green.wat` — UNCHANGED by this pass (the `pure`
//!     flip, `d6c32cf5`'s own fix). Still GREEN.
//!   - `probe_constructor_meta_surface_total_aggregate.wat` — #1. USED TO prove the aggregate
//!     site's `total: false` (a nested surface constructor compiled clean, died at fire with
//!     `UnknownFunction`). NOW proves the fix: `dispatch_keyword_head_value` wires a bare
//!     aggregate-type keyword to `eval_kwargs_construct`, so this SAME rule compiles AND fires,
//!     via both the oracle and the native kernel.
//!   - `probe_constructor_meta_surface_total_enum.wat.bad` — #3. USED TO prove the enum-variant
//!     site's `total: false` (a wrong-arity nested variant call compiled clean, died at fire
//!     with `ArityMismatch`). NOW must FAIL `startup_from_file` — `walk_nested_constructors`
//!     resolves the bare `:Enum::Variant` head and walls the arity at freeze. Renamed `.wat` →
//!     `.wat.bad` (a should-never-start-up fixture, the repo's own convention for that) since
//!     its assertion inverted from "starts up clean" to "must not start up".
//!   - `probe_constructor_meta_enum_variant_green.wat` — NEW, #3's "STILL WORKS" half: a
//!     CORRECT-arity nested enum-variant call must not be rejected by the new wall. Oracle +
//!     native.
//!   - `probe_constructor_meta_kwargs_undersupply.wat.bad` — NEW, #2's "REJECT" half: a
//!     top-level kwargs `:then` item supplying fewer than all of its type's declared fields must
//!     now fail `startup_from_file` with a located `RhsMissingFields`, naming the missing field.
//!     STOP-A (the corpus audit BRIEF-construction-total-three-walls.md required before closing
//!     #2) found no `:then` in the corpus relying on the old under-supply.
//!   - `probe_constructor_meta_kwargs_full_green.wat` — NEW, #2's "STILL WORKS" half: the same
//!     shape, fully supplied, must still compile and fire. Oracle + native.
//!
//! Run: cargo test --release -p wat --test rete probe_constructor_meta_surface_audit

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

const WORLD_PURE_GREEN: &str = "tests/rete/probe_constructor_meta_surface_pure_green.wat";
const WORLD_NESTED_AGGREGATE_GREEN: &str =
    "tests/rete/probe_constructor_meta_surface_total_aggregate.wat";
const WORLD_ENUM_ARITY_BAD: &str = "tests/rete/probe_constructor_meta_surface_total_enum.wat.bad";
const WORLD_ENUM_VARIANT_GREEN: &str = "tests/rete/probe_constructor_meta_enum_variant_green.wat";
const WORLD_KWARGS_UNDERSUPPLY_BAD: &str =
    "tests/rete/probe_constructor_meta_kwargs_undersupply.wat.bad";
const WORLD_KWARGS_FULL_GREEN: &str = "tests/rete/probe_constructor_meta_kwargs_full_green.wat";

/// Run the named zero-arg entry fn and return its result, or an `Err` string for either an
/// ordinary raise OR the fence's `Option/expect` compile-time panic — the same dual capture
/// `probe_construction_headline.rs::run` uses, since a regression on the `pure` fix would
/// surface as a PANIC during `(:wat::rete::compile rules)`, not a clean `Err`.
fn run(world_path: &str, fn_name: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(world_path)?;
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no entry fn {fn_name:?} in {world_path}"))
        .clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| StartupError::Runtime(Box::new(e))),
        Err(panic_payload) => {
            let message = panic_payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "panic-opaque".to_string());
            Err(StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::AssertionFailed { message, actual: None, expected: None },
            ))))
        }
    }
}

/// PURE flip (`d6c32cf5`, UNCHANGED by this pass): a `Nature::Struct` constructed via its bare
/// surface form directly in a `:then` item now compiles (no axis-violation panic during
/// `compile-rule`) AND fires, returning the value carried through the struct's only field.
#[test]
fn struct_surface_constructor_now_admitted_pure() {
    let r = run(WORLD_PURE_GREEN, ":user::run");
    assert!(matches!(r, Ok(Value::i64(5))), "expected label=5 via the newly-admitted surface struct constructor; got {r:?}");
}

/// #1, RE-POINTED (used to assert a fire-time `UnknownFunction`, `d6c32cf5`): a nested surface
/// aggregate constructor now WORKS end to end — `dispatch_keyword_head_value` wires a bare
/// aggregate-type keyword to the SAME `eval_kwargs_construct` dispatch the macro-expanded form
/// uses. Oracle path.
#[test]
fn nested_surface_aggregate_constructor_now_works_via_oracle() {
    let r = run(WORLD_NESTED_AGGREGATE_GREEN, ":user::run-oracle");
    assert!(matches!(r, Ok(Value::i64(5))), "expected Inner.x=5 via the newly-wired nested constructor (oracle); got {r:?}");
}

/// #1, native-kernel counterpart — same rule, compiled RHS path (`insert`/`fire-rules`).
#[test]
fn nested_surface_aggregate_constructor_now_works_via_native_kernel() {
    let r = run(WORLD_NESTED_AGGREGATE_GREEN, ":user::run-native");
    assert!(matches!(r, Ok(Value::i64(5))), "expected Inner.x=5 via the newly-wired nested constructor (native); got {r:?}");
}

/// #3, RE-POINTED (used to assert a clean fire-time `ArityMismatch`, `d6c32cf5`): a wrong-arity
/// nested enum-variant call is now a FREEZE-time rejection — `startup_from_file` itself must
/// fail, naming the rule, the full variant path, and the actual/expected arity.
#[test]
fn nested_surface_enum_variant_wrong_arity_is_now_a_check_time_rejection() {
    let err = startup_from_file(WORLD_ENUM_ARITY_BAD).expect_err(
        "a wrong-arity nested enum-variant call must fail to start up (checker rejection), not compile clean and die at fire",
    );
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — the Debug rendering embeds an absolute file path (Span),
    // non-deterministic across machines/CI — assert the load-bearing SUBSTANCE, not the blob.
    assert!(msg.contains("RhsArityMismatch"), "wrong error kind, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("cg::gather"), "error does not name the rule, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("cg::Status::Active"), "error does not name the offending variant, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("expected 1") && msg.contains("got 3"), "error does not name the actual/expected arity, got:\n{msg}");
}

/// #3's "STILL WORKS" half: a CORRECT-arity nested enum-variant call must not be rejected by the
/// new wall. Oracle path.
#[test]
fn nested_surface_enum_variant_correct_arity_still_works_via_oracle() {
    let r = run(WORLD_ENUM_VARIANT_GREEN, ":user::run-oracle");
    assert!(matches!(r, Ok(Value::i64(7))), "expected Active.level=7 (oracle); got {r:?}");
}

/// #3's "STILL WORKS" half, native-kernel counterpart.
#[test]
fn nested_surface_enum_variant_correct_arity_still_works_via_native_kernel() {
    let r = run(WORLD_ENUM_VARIANT_GREEN, ":user::run-native");
    assert!(matches!(r, Ok(Value::i64(7))), "expected Active.level=7 (native); got {r:?}");
}

/// #2, NEW: a top-level kwargs `:then` item supplying fewer than all of its type's declared
/// fields is now a FREEZE-time rejection (`RhsMissingFields`), naming the rule, the type, and
/// the missing field.
#[test]
fn kwargs_undersupply_is_now_a_check_time_rejection() {
    let err = startup_from_file(WORLD_KWARGS_UNDERSUPPLY_BAD).expect_err(
        "an under-supplied kwargs :then item must fail to start up (checker rejection), not silently build a short record",
    );
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — same span/path reason as the enum-arity test above.
    assert!(msg.contains("RhsMissingFields"), "wrong error kind, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("cr2::gather"), "error does not name the rule, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("cr2::Rate"), "error does not name the fact type, got:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason.
    assert!(msg.contains("window"), "error does not name the missing field, got:\n{msg}");
}

/// #2's "STILL WORKS" half: a fully-supplied kwargs `:then` item must not be rejected by the new
/// wall. Oracle path.
#[test]
fn kwargs_full_supply_still_works_via_oracle() {
    let r = run(WORLD_KWARGS_FULL_GREEN, ":user::run-oracle");
    assert!(matches!(r, Ok(Value::i64(16))), "expected count(7)+window(9)=16 (oracle); got {r:?}");
}

/// #2's "STILL WORKS" half, native-kernel counterpart.
#[test]
fn kwargs_full_supply_still_works_via_native_kernel() {
    let r = run(WORLD_KWARGS_FULL_GREEN, ":user::run-native");
    assert!(matches!(r, Ok(Value::i64(16))), "expected count(7)+window(9)=16 (native); got {r:?}");
}
