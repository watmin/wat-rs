//! Arc 255.1b-iv-c — disconfirming probe: `metadata-of` emits PLAIN wat
//! values (and `Value::Enum` for the closed-domain fields), NOT holon-AST.
//!
//! THE DEFECT (caught by dogfooding the reflection surface as EDN):
//! `eval_metadata_of`'s intrinsic branch wraps every value in
//! `Value::holon__HolonAST` (`runtime.rs` ~10111 `put` closure). So the
//! metadata map EDN-serializes with holon-algebra tags — the holon VSA algebra-AST encoder leaking
//! into reflection. This is the same `HolonAST`-as-`EdnRepresentable` crutch
//! the codebase has been rooting out (`impl EdnRepresentable for Value`,
//! comms/mod.rs:794 — plain values already serialize cleanly).
//!
//! THE CONTRACT — RE-DIAGNOSED, arc 255 Stone P3 (2026-08-28). The `:pure` /
//! `:deterministic` PLAIN-bool contract this test used to assert was never what
//! shipped. `runtime.rs`'s `eval_metadata_of` intrinsic branch (~13716-13719) puts
//! `:purity` / `:determinism` (not `:pure` / `:deterministic`) as DECLARED
//! `Value::Enum`s parsed from the doc (`wat_doc::Purity`/`Determinism`), not derived
//! bools — see `src/intrinsic/mod.rs`'s header, "CORRECTED 2026-08-25". Verified live
//! this session (`wat-scripts/scratch-pad/255-p3/dump-to-hex-metadata.wat`):
//! `metadata-of(:wat::core::Bytes::to-hex)` puts `:purity #wat.runtime.Purity/Pure`
//! and `:determinism #wat.runtime.Determinism/Deterministic` — enums, not bools.
//!
//! THE CONTRACT (as shipped):
//!  - The baseline scalar fields are PLAIN wat values:
//!    :name -> Value::wat__core__keyword   :arity -> Value::i64
//!    :doc/:added/:ret -> Value::String
//!  - FIVE closed-domain fields are `Value::Enum` (typo-proof; backed by
//!    wat `defenum` + Rust enum mirror), rendering to EDN as a qualified
//!    keyword (`:wat.runtime.Kind/Intrinsic`):
//!    :kind        -> Value::Enum :wat::runtime::Kind / Intrinsic
//!    :defined-in  -> Value::Enum :wat::runtime::DefinedIn / Rust
//!    :layer       -> Value::Enum :wat::runtime::Layer / Substrate
//!    :purity      -> Value::Enum :wat::runtime::Purity / Pure
//!    :determinism -> Value::Enum :wat::runtime::Determinism / Deterministic
//!  - NO value in the map is `Value::holon__HolonAST` (the cross-cutting RED this
//!    probe was written to catch, and which iv-c fixed).
//!
//! RED at HEAD (pre-255.1b-iv-c): every value was `Value::holon__HolonAST`.
//! GREEN (as shipped): plain values + the five enums above.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// just-eval (rubric): the metadata-of(:wat::core::Bytes::to-hex) call lives in
/// the co-located fixture (`:user::to-hex-metadata`), driven via `call_beside_value`;
/// the Rust side inspects the returned `Some(HashMap)`. The `_fqdn` arg is kept
/// for call-site readability (the fixture pins the single fqdn under test).
fn metadata_of(_fqdn: &str) -> std::collections::HashMap<Value, Value> {
    match call_beside_value(file!(), ":user::to-hex-metadata").expect("eval metadata-of") {
        Value::Option(opt) => match &*opt {
            Some(Value::wat__std__HashMap(m)) => (**m).clone(),
            other => panic!("metadata-of must be Some(HashMap); got {other:?}"),
        },
        other => panic!("metadata-of must return Option; got {other:?}"),
    }
}

/// Look a `:keyword` key up in the metadata map.
fn get<'a>(map: &'a std::collections::HashMap<Value, Value>, key: &str) -> &'a Value {
    map.iter()
        .find_map(|(k, v)| match k {
            Value::wat__core__keyword(s) if s.as_str() == key => Some(v),
            _ => None,
        })
        .unwrap_or_else(|| panic!("metadata map missing key {key}"))
}

/// Assert a value is a unit `Value::Enum` of the given type_path + variant.
fn assert_enum(v: &Value, type_path: &str, variant: &str) {
    match v {
        Value::Enum(ev) => {
            assert_eq!(ev.type_path, type_path, "enum type_path");
            assert_eq!(ev.variant_name, variant, "enum variant_name");
            assert!(ev.fields.is_empty(), "closed-domain field is a UNIT variant");
        }
        other => panic!("expected Value::Enum({type_path}/{variant}); got {other:?}"),
    }
}

#[test]
fn metadata_of_emits_plain_values_and_enums_not_holon_ast() {
    let map = metadata_of(":wat::core::Bytes::to-hex");

    // The five closed-domain fields are enums (as shipped — see the module doc
    // above for the re-diagnosis; :purity/:determinism replaced the never-shipped
    // :pure/:deterministic plain-bool contract this test used to assert).
    assert_enum(get(&map, ":kind"), ":wat::runtime::Kind", "Intrinsic");
    assert_enum(get(&map, ":defined-in"), ":wat::runtime::DefinedIn", "Rust");
    assert_enum(get(&map, ":layer"), ":wat::runtime::Layer", "Substrate");
    assert_enum(get(&map, ":purity"), ":wat::runtime::Purity", "Pure");
    assert_enum(get(&map, ":determinism"), ":wat::runtime::Determinism", "Deterministic");

    // The baseline scalars are PLAIN wat values (not holon-AST-wrapped).
    assert!(
        matches!(get(&map, ":name"), Value::wat__core__keyword(_)),
        ":name must be a plain keyword"
    );
    assert!(matches!(get(&map, ":arity"), Value::i64(_)), ":arity must be a plain i64");
    assert!(matches!(get(&map, ":doc"), Value::String(_)), ":doc must be a plain String");

    // The cross-cutting RED: NOT ONE value rides as holon AST.
    for (k, v) in &map {
        assert!(
            !matches!(v, Value::holon__HolonAST(_)),
            "metadata value for {k:?} is still holon__HolonAST — the encoder leak iv-c removes"
        );
    }
}
