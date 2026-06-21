//! Arc 255.1b-iv-b2-a — disconfirming probe: the `:wat::intrinsic::examples`
//! reflection seam exposes the carried doc examples to wat.
//!
//! THE ASK (R2, the self-hosting verifier): a thin Rust seam reads each registered
//! intrinsic's carried `@example`s off the registry, parses each example's `expr`
//! (and `#=> expected`) into a quoted form, and returns them to wat as data —
//! mirroring `:wat::stdlib::sources` → `verify-stdlib`. The wat verifier
//! (`verify-examples`, iv-b2-b) consumes this. This seam is also the READER that
//! retires iv-b1's `#[expect(dead_code)]` on `IntrinsicEntry.examples`.
//!
//! Tuple contract per element: `[fqdn, expr, expected, run, pure, deterministic]`
//! — `fqdn` a keyword, `expr`/`expected` quoted forms (`expected` nil for a
//! markerless `@example-norun`), `run`/`pure`/`deterministic` bools.
//!
//! RED AT HEAD: `:wat::intrinsic::examples` has no dispatch arm → the call errors
//! at runtime (the resolver blanket-accepts the `:wat::*` head, but nothing
//! handles it). GREEN after b2-a: a `Vector` of tuples including
//! `:wat::core::Bytes::to-hex` with `run = true`.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Freeze a nil-main world and eval `(:wat::intrinsic::examples)`, returning the
/// value or a rendered error. RED at HEAD = `Err` (no dispatch arm for the seam).
fn eval_examples_seam() -> Result<Value, String> {
    let main = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(main, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one_with_file("(:wat::intrinsic::examples)", "<probe>")
        .map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|s| s.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

#[test]
fn examples_seam_returns_bytes_to_hex_runnable() {
    // RED at HEAD: the seam has no handler → eval errors here.
    let v = eval_examples_seam()
        .expect("(:wat::intrinsic::examples) must eval to a Vector of example tuples");

    let entries = match v {
        Value::Vec(xs) => xs,
        other => panic!("seam must return a Vector; got {:?}", other),
    };
    assert!(!entries.is_empty(), "seam must return at least one example");

    // Find the to-hex tuple (colon-normalized fqdn match, robust to keyword convention).
    let to_hex = entries
        .iter()
        .filter_map(|e| match e {
            Value::Vec(cols) => Some(cols),
            _ => None,
        })
        .find(|cols| match cols.first() {
            Some(Value::wat__core__keyword(k)) => {
                k.trim_start_matches(':') == "wat::core::Bytes::to-hex"
            }
            _ => false,
        })
        .expect("seam must include the :wat::core::Bytes::to-hex example");

    // Tuple contract: [fqdn, expr, expected, run, pure, det] — run at index 3.
    assert!(
        to_hex.len() >= 4,
        "example tuple must carry at least [fqdn, expr, expected, run]; got {} cols",
        to_hex.len()
    );
    assert_eq!(
        to_hex.get(3),
        Some(&Value::bool(true)),
        "to-hex's @example is runnable (run = true at tuple index 3)"
    );
}
