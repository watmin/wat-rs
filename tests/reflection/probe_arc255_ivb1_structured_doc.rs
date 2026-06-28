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
use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

/// Freeze a bare world and eval `(metadata-of <name_kw>)`; return whether the
/// resulting `Some(HashMap)` contains the keyword key `key` (e.g. ":added").
fn metadata_of_has_key(name_kw: &str, key: &str) -> bool {
    let world = startup_bare().expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
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
        metadata_of_has_key(":wat::core::Bytes::to-hex", ":added"),
        "metadata-of for a fully-contracted intrinsic must carry :added (from @added). \
         RED at HEAD: Bytes is prose-only and the macro emits only :doc + baseline. \
         GREEN after iv-b1: the macro parses the `///` via wat-doc and carries the structured doc."
    );
}

// RED at HEAD: prose-only `///` -> no `:ret` in the metadata map.
#[test]
fn bytes_to_hex_metadata_carries_ret() {
    assert!(
        metadata_of_has_key(":wat::core::Bytes::to-hex", ":ret"),
        "metadata-of for a fully-contracted intrinsic must carry :ret (from @ret). \
         RED at HEAD (prose-only); GREEN after iv-b1 (structured carry)."
    );
}
