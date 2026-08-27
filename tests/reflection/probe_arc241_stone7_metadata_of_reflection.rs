//! FM 2-bis probe for Stone 241.7 — mint `:wat::runtime::metadata-of` reflection verb.
//!
//! Reads SymbolTable.binding_metadata that Stone 241.6 stored. Returns
//! Option<HashMap<Keyword, HolonAST>> per arc 216.7 + 218.2 FQDN tagged-literal
//! encoding (`#wat.core/Some {...}` / `#wat.core/None nil`).
//!
//! Pre-stone: contracts FAIL — the verb doesn't exist; calls error.
//! Post-stone: N/N PASS; reflection round-trips metadata stored by 241.6.
//!
//! Run: `cargo test --release --test probe_arc241_stone7_metadata_of_reflection`

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

// just-eval (rubric): each `*_cNN.wat` fixture defines a zero-arg `:user::compute`;
// fetch it from the frozen world and `apply_function` it — no inline wat driver.
// (Path-based rather than `call_beside_value` because this probe shares one `.rs` across
// five co-located fixtures, so the fixture is not the single sibling `.wat`.)
fn compute_from_file(fixture: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(fixture)?;
    let func = world
        .symbols()
        .get(":user::compute")
        .ok_or_else(|| {
            StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::UnboundSymbol(":user::compute".to_string()),
            )))
        })?
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))
}

fn is_some(v: &Value) -> bool {
    matches!(v, Value::Option(opt) if opt.as_ref().is_some())
}

fn is_none(v: &Value) -> bool {
    matches!(v, Value::Option(opt) if opt.as_ref().is_none())
}

// ─── Contracts 1–3: presence path (Some) ─────────────────────────────────────

#[test]
fn contract_01_def_with_metadata_returns_some() {
    let result = compute_from_file("tests/reflection/probe_arc241_stone7_metadata_of_reflection_c01.wat")
        .expect("def-with-metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "def-with-metadata metadata-of returns Some; got: {:?}",
        result
    );
}

#[test]
fn contract_02_defn_with_metadata_returns_some() {
    let result = compute_from_file("tests/reflection/probe_arc241_stone7_metadata_of_reflection_c02.wat")
        .expect("defn-with-metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "defn-with-metadata metadata-of returns Some via fn-peel round-trip; got: {:?}",
        result
    );
}

#[test]
fn contract_03_multi_entry_metadata_returns_some() {
    let result = compute_from_file("tests/reflection/probe_arc241_stone7_metadata_of_reflection_c03.wat")
        .expect("multi-entry metadata metadata-of must not error");
    assert!(
        is_some(&result),
        "multi-entry metadata round-trips via Some; got: {:?}",
        result
    );
}

// ─── Contracts 4–5: absence path (None) ──────────────────────────────────────

#[test]
fn contract_04_def_without_metadata_returns_none() {
    let result = compute_from_file("tests/reflection/probe_arc241_stone7_metadata_of_reflection_c04.wat")
        .expect("def-without-metadata metadata-of must not error");
    assert!(
        is_none(&result),
        "def-without-metadata metadata-of returns None; got: {:?}",
        result
    );
}

#[test]
fn contract_05_unknown_binding_returns_none() {
    // Unknown name -> None (not an error).
    let result = compute_from_file("tests/reflection/probe_arc241_stone7_metadata_of_reflection_c05.wat")
        .expect("unknown binding metadata-of must not error");
    assert!(
        is_none(&result),
        "unknown binding metadata-of returns None; got: {:?}",
        result
    );
}
