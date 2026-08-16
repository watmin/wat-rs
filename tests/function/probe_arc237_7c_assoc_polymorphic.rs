//! FM-2-bis probe for Stone 237.7c — settle the polymorphic `assoc` recipe BEFORE
//! briefing the alias-to-intrinsic promotion.
//!
//! `:wat::core::assoc` today is a `define-alias` (HashMap-only; arc 146 slice 4,
//! `wat/core.wat:50`). `:wat::core::Record/assoc` exists separately (arc 234.3b) and
//! already accepts both base + holonic records (Liskov; flavor-preserving via
//! the early-return base arm + holonic fallthrough at runtime.rs:17129).
//!
//! Stone 237.7c promotes the surface name to a Rust ∀T intrinsic with a custom
//! inference arm spanning HashMap + Record (the records-doctrine slice the
//! `DESIGN-STONE-237.7b.md` flagged at line 96).
//!
//! ROW STATUS:
//!   - 4 rows GREEN AT HEAD `e435194d`+ (regression contract — HashMap path
//!     works through the alias; non-collection arg0 already errors).
//!   - 2 rows `#[ignore]`d AT HEAD (disconfirming: the Record arms FAIL today
//!     because the alias is HashMap-only). Sonnet's stone work MUST remove the
//!     `#[ignore]` annotations as part of the sweep — after the intrinsic is
//!     wired, both rows go GREEN. The un-ignore is the contract.
//!
//! Run: cargo test --release --test probe_arc237_7c_assoc_polymorphic

//! Wat source: tests/function/probe_arc237_7c_assoc_polymorphic.wat
//! Negative fixtures: probe_arc237_7c_wrong_key.wat.bad, probe_arc237_7c_wrong_value.wat.bad,
//!   probe_arc237_7c_non_collection.wat.bad.
//! Ignored Record-arm fixtures: probe_arc237_7c_assoc_base_record.wat,
//!   probe_arc237_7c_assoc_holonic_record.wat (un-ignored when Stone 237.7c ships).

use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for 7c assoc-polymorphic fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

/// Fetch + apply `:user::compute` from an explicitly-loaded (non-sibling) fixture path —
/// the two Record-arm probes below each own a dedicated fixture, not the co-located
/// `.wat` `startup_beside` slurps by default.
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup (RED until Stone 237.7c ships)");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval")
}

// ─── HashMap arm — regression contract ────────────────────────────────────────

#[test]
fn assoc_hashmap_returns_hashmap_type_preserved() {
    // `(assoc m "k" 1)` returns a HashMap; HashMap/keys returns keys of length 1.
    assert_eq!(
        run(":user::assoc-hashmap"),
        Value::i64(1),
        "assoc HashMap returns HashMap; keys returns Vec<String> of length 1",
    );
}

#[test]
fn assoc_hashmap_wrong_key_type_rejected_at_check() {
    let result = startup_from_file("tests/function/probe_arc237_7c_wrong_key.wat.bad");
    assert!(result.is_err(), "assoc HashMap<String,i64> with i64 key MUST reject at check; got Ok");
}

#[test]
fn assoc_hashmap_wrong_value_type_rejected_at_check() {
    let result = startup_from_file("tests/function/probe_arc237_7c_wrong_value.wat.bad");
    assert!(result.is_err(), "assoc HashMap<String,i64> with String value MUST reject at check; got Ok");
}

#[test]
fn assoc_non_collection_arg0_rejected() {
    let result = startup_from_file("tests/function/probe_arc237_7c_non_collection.wat.bad");
    assert!(result.is_err(), "assoc with non-collection arg0 (i64) MUST reject; got Ok");
}

// ─── Record arm — disconfirming AT HEAD; un-ignore in Stone 237.7c ─────────────────

#[test]
// UN-IGNORED 2026-08-16: Stone 237.7c SHIPPED `a9961421` (2026-05-27) — eval_record_assoc
// is live at src/runtime.rs:4834 with both base and holonic arms. 81 days stale.
fn assoc_base_record_returns_base_record_struct_only() {
    // POST-7c contract: assoc rebuilds the base record with the field updated.
    match compute_from_file("tests/function/probe_arc237_7c_assoc_base_record.wat") {
        Value::i64(n) => assert_eq!(n, 42, "assoc on base record updates the field"),
        other => panic!("expected i64(42); got {:?}", other),
    }
}

#[test]
// UN-IGNORED 2026-08-16: see the sibling above — 237.7c shipped `a9961421`.
fn assoc_holonic_record_returns_holonic_record_parity_preserved() {
    // POST-7c contract: assoc rebuilds BOTH struct_form AND holon_form in parity.
    match compute_from_file("tests/function/probe_arc237_7c_assoc_holonic_record.wat") {
        Value::i64(n) => assert_eq!(n, 42, "assoc on holonic record updates the field (parity rebuilt)"),
        other => panic!("expected i64(42); got {:?}", other),
    }
}
