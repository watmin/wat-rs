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
//! Record contract per element: a `:wat::intrinsic::Example` record with fields
//! (declaration order) `[fqdn, expr, expected, run, pure, deterministic]` —
//! `fqdn` a keyword, `expr`/`expected` quoted forms (`expected` nil for a
//! markerless/`@example-norun`), `run`/`pure`/`deterministic` bools. Records (not
//! heterogeneous tuples) because R7's unidirectional `Value` makes a tuple's
//! universal-top elements un-passable to the typed `eval-ast!` (the firewall, R3).
//!
//! RED until the records rework: the shipped seam returns `Value::Vec` tuples, so
//! the `Value::Struct` match below finds nothing. GREEN after: a `Vector` of
//! `:wat::intrinsic::Example` records including `:wat::core::Bytes::to-hex`
//! (`run = true`).

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
    // RED until the seam exists / returns records; the eval errors or the
    // Value::Struct match finds nothing.
    let v = eval_examples_seam()
        .expect("(:wat::intrinsic::examples) must eval to a Vector of Example records");

    let entries = match v {
        Value::Vec(xs) => xs,
        other => panic!("seam must return a Vector; got {:?}", other),
    };
    assert!(!entries.is_empty(), "seam must return at least one example");

    // Find the to-hex Example RECORD (Value::Struct of :wat::intrinsic::Example),
    // colon-normalized fqdn match. RED on the tuple seam (elements are Value::Vec).
    let to_hex = entries
        .iter()
        .filter_map(|e| match e {
            Value::Struct(s) if s.type_name == ":wat::intrinsic::Example" => Some(s),
            _ => None,
        })
        .find(|s| match s.fields.first() {
            Some(Value::wat__core__keyword(k)) => {
                k.trim_start_matches(':') == "wat::core::Bytes::to-hex"
            }
            _ => false,
        })
        .expect("seam must return :wat::intrinsic::Example records including Bytes::to-hex");

    // Field order = declaration order: [fqdn, expr, expected, run, pure, det] — run at index 3.
    assert!(
        to_hex.fields.len() >= 4,
        "Example record must carry at least [fqdn, expr, expected, run]; got {} fields",
        to_hex.fields.len()
    );
    assert_eq!(
        to_hex.fields.get(3),
        Some(&Value::bool(true)),
        "to-hex's @example is runnable (run = true at field index 3)"
    );
}
