//! FM-2-bis probe for Stone 237.7c — settle the polymorphic `assoc` recipe BEFORE
//! briefing the alias-to-intrinsic promotion.
//!
//! `:wat::core::assoc` today is a `define-alias` (HashMap-only; arc 146 slice 4,
//! `wat/core.wat:50`). `:wat::Record/assoc` exists separately (arc 234.3b) and
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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn eval_value(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── HashMap arm — regression contract (works today via alias; works post via intrinsic) ────

#[test]
fn assoc_hashmap_returns_hashmap_type_preserved() {
    // `(assoc m "k" 1)` returns a HashMap usable by collection ops downstream.
    // Feeds the result into `:wat::core::HashMap/keys` (per-Type leaf, untouched
    // by the stone) — proves the return is still typed HashMap<String, i64>.
    assert_eq!(
        eval_value(
            r#"(:wat::core::defn :user::compute [] -> :wat::core::i64
              (:wat::core::length
                                 (:wat::core::HashMap/keys
                                   (:wat::core::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "k" 1))))"#
        ),
        Value::i64(1),
        "assoc HashMap returns HashMap; keys returns Vec<String> of length 1",
    );
}

#[test]
fn assoc_hashmap_wrong_key_type_rejected_at_check() {
    // HashMap<String, i64>; pass an i64 as the key — must reject at check time
    // via the K-type discipline (today via the alias's HashMap/assoc scheme;
    // post via `infer_assoc` HashMap arm).
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::length
                         (:wat::core::HashMap/keys
                           (:wat::core::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) 42 1))))"#,
    );
    assert!(
        result.is_err(),
        "assoc HashMap<String,i64> with i64 key MUST reject at check; got: {:?}",
        result,
    );
}

#[test]
fn assoc_hashmap_wrong_value_type_rejected_at_check() {
    // HashMap<String, i64>; pass a String as the value — must reject via V-type
    // discipline.
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::length
                         (:wat::core::HashMap/keys
                           (:wat::core::assoc (:wat::core::HashMap :wat::core::String :wat::core::i64) "k" "v"))))"#,
    );
    assert!(
        result.is_err(),
        "assoc HashMap<String,i64> with String value MUST reject at check; got: {:?}",
        result,
    );
}

#[test]
fn assoc_non_collection_arg0_rejected() {
    // Pass an i64 as arg0 — neither HashMap nor Record. Today the alias's
    // HashMap-only scheme rejects at check; post, `infer_assoc`'s else-arm
    // returns a teaching TypeMismatch. Either way: not green.
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::length
                         (:wat::core::HashMap/keys
                           (:wat::core::assoc 42 "k" 1))))"#,
    );
    assert!(
        result.is_err(),
        "assoc with non-collection arg0 (i64) MUST reject; got: {:?}",
        result,
    );
}

// ─── Record arm — disconfirming AT HEAD; un-ignore in Stone 237.7c ─────────────────

#[test]
fn assoc_base_record_returns_base_record_struct_only() {
    // Mint a 1-field defrecord, instantiate, assoc a new value, read back via
    // the auto-accessor. Base record (no holon_form) — assoc rebuilds struct only.
    //
    // POST-7c contract: `(:wat::core::assoc rec :name "new")` works; the result
    // is still a `:my::Voltage` base record with the field updated.
    assert_eq!(
        eval_value(
            r#"(:wat::core::defrecord :my::Voltage [value <- :wat::core::i64])
               (:wat::core::defn :user::compute [] -> :wat::core::i64
                 (:my::Voltage/value
                                    (:wat::core::assoc (:my::Voltage 10) :value 42)))"#
        ),
        Value::i64(42),
        "assoc on base record updates the field; accessor reads the new value",
    );
}

#[test]
fn assoc_holonic_record_returns_holonic_record_parity_preserved() {
    // Same shape but holonic (`:wat::holon::defrecord`). The holonic arm in
    // `eval_record_assoc` rebuilds BOTH struct_form AND holon_form in parity.
    // The post-7c intrinsic must route through the same path, preserving flavor.
    //
    // We probe by reading back through the auto-accessor (the field read goes
    // through the struct path; if assoc broke parity, the result would still
    // be 42 — but if the intrinsic's eval arm dropped the holonic flavor, the
    // record would degrade to base and other holon-ops would fail downstream).
    // For the probe, the load-bearing assertion is the round-trip i64.
    assert_eq!(
        eval_value(
            r#"(:wat::holon::defrecord :my::HolonicVoltage [value <- :wat::core::i64])
               (:wat::core::defn :user::compute [] -> :wat::core::i64
                 (:my::HolonicVoltage/value
                                    (:wat::core::assoc (:my::HolonicVoltage 10) :value 42)))"#
        ),
        Value::i64(42),
        "assoc on holonic record updates the field; accessor reads the new value (parity rebuilt)",
    );
}
