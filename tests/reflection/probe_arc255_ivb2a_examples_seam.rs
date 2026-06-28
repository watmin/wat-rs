//! Arc 255.1b-iv-b2-a — disconfirming probe: the `:wat::intrinsic::examples`
//! reflection seam exposes the carried doc examples to wat.
//!
//! THE ASK (R2, the self-hosting verifier): a thin Rust seam reads each registered
//! intrinsic's carried `@example`s off the registry, parses each example's `expr`
//! (and `#=> expected`) into a quoted form, and returns them to wat as data —
//! mirroring `:wat::stdlib::sources` -> `verify-stdlib`. The wat verifier
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
//! The records are `Value::wat__Record` (the `:wat::core::defrecord` representation —
//! EDN-representable data is a wat-record, not a `Value::Struct`; that's the
//! builder doctrine, and it's what makes the named field accessors work). GREEN:
//! a `Vector` of `:wat::intrinsic::Example` records including
//! `:wat::core::Bytes::to-hex` (`run = true`).

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};
use wat::types::Holder;

/// Freeze a bare world and eval `(:wat::intrinsic::examples)`, returning the
/// value or a rendered error. RED at HEAD = `Err` (no dispatch arm for the seam).
fn eval_examples_seam() -> Result<Value, String> {
    let world = startup_bare().map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one_with_file("(:wat::intrinsic::examples)", "<probe>")
        .map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|s| s.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

#[test]
fn examples_seam_returns_bytes_to_hex_runnable() {
    // RED until the seam returns wat__Record Example values; the eval errors or
    // the wat__Record match below finds nothing.
    let v = eval_examples_seam()
        .expect("(:wat::intrinsic::examples) must eval to a Vector of Example records");

    let entries = match v {
        Value::Vec(xs) => xs,
        other => panic!("seam must return a Vector; got {:?}", other),
    };
    assert!(!entries.is_empty(), "seam must return at least one example");

    // Find the to-hex Example — a Value::wat__Record of class `wat::intrinsic::Example`
    // (EDN-representable data is a wat-record, NOT a Value::Struct — builder doctrine).
    // RED on the tuple seam (elements were Value::Vec); RED on a Value::Struct seam too.
    let to_hex = entries
        .iter()
        .filter_map(|e| match e {
            Value::Aggregate(a)
                if a.holder != Holder::Struct && a.class == "wat::intrinsic::Example" =>
            {
                Some(&a.fields)
            }
            _ => None,
        })
        .find(|sf| match sf.first() {
            Some(Value::wat__core__keyword(k)) => {
                k.trim_start_matches(':') == "wat::core::Bytes::to-hex"
            }
            _ => false,
        })
        .expect("seam must return wat__Record :wat::intrinsic::Example values including Bytes::to-hex");

    // Field order = declaration order: [fqdn, expr, expected, run, pure, det] — run at index 3.
    assert!(
        to_hex.len() >= 4,
        "Example record must carry at least [fqdn, expr, expected, run]; got {} fields",
        to_hex.len()
    );
    assert_eq!(
        to_hex.get(3),
        Some(&Value::bool(true)),
        "to-hex's @example is runnable (run = true at field index 3)"
    );
}
