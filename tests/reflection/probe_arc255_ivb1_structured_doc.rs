//! Arc 255.1b-iv-b1 — disconfirming probe: the registry entry carries the
//! STRUCTURED doc, and `metadata-of` renders it.
//!
//! THE ASK: once `#[wat_intrinsic]` parses the `///` through `wat-doc` and carries
//! the structured directives on the registry entry, `metadata-of` for a fully-
//! contracted intrinsic must answer with the richer keys — at minimum `:added`
//! (from `@added`) and `:ret` (from `@ret`) — not just the `:doc` prose blob.
//!
//! RED AT HEAD:
//!   - `core::Bytes::to-hex` carries a prose-only `///`; the macro `sniff_doc`s the
//!     whole string into `:doc`. `metadata-of` emits `:doc` + the derived baseline
//!     ONLY — no `:added`, no `:ret`.
//! GREEN AFTER iv-b1: Bytes decorated to the full contract -> the macro parses +
//!   carries the structured doc -> `metadata-of` emits `:added` and `:ret`.

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// just-eval (rubric): the metadata-of call lives in the co-located fixture
/// (`:user::to-hex-metadata`), driven via `call_beside_value`. Returns whether the
/// resulting `Some(HashMap)` contains the keyword key `key` (e.g. ":added") —
/// the same containment assertion the format!-string driver made.
fn metadata_of_has_key(key: &str) -> bool {
    match call_beside_value(file!(), ":user::to-hex-metadata").expect("metadata-of eval") {
        Value::Option(o) => match &*o {
            Some(Value::wat__std__HashMap(m)) => {
                let k = Value::wat__core__keyword(Arc::new(key.to_string()));
                m.contains_key(&k)
            }
            Some(other) => panic!("metadata-of must wrap a HashMap; got {:?}", other),
            None => false,
        },
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

// RED at HEAD: prose-only `///` -> no `:added` in the metadata map.
#[test]
fn bytes_to_hex_metadata_carries_added() {
    assert!(
        metadata_of_has_key(":added"),
        "metadata-of for a fully-contracted intrinsic must carry :added (from @added). \
         RED at HEAD: Bytes is prose-only and the macro emits only :doc + baseline. \
         GREEN after iv-b1: the macro parses the `///` via wat-doc and carries the structured doc."
    );
}

// RED at HEAD: prose-only `///` -> no `:ret` in the metadata map.
#[test]
fn bytes_to_hex_metadata_carries_ret() {
    assert!(
        metadata_of_has_key(":ret"),
        "metadata-of for a fully-contracted intrinsic must carry :ret (from @ret). \
         RED at HEAD (prose-only); GREEN after iv-b1 (structured carry)."
    );
}
