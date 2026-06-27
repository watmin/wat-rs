//! Arc 118.2 — DISCONFIRMING PROBE: `:wat::core::map` is LAZY (does not force the whole input).
//!
//! Contract (`118.2/DESIGN`): `core::map` flips to LAZY — it returns a `Stream` and applies its
//! fn only as the consumer pulls. So mapping a fn that errors on a LATE element, then pulling only
//! the HEAD, must NOT hit the late error.
//!
//! `boom` errors only on `99` (the 3rd element); `:my::compute` maps it over `[1 2 99]` and returns
//! only `(first …)`. NOTE: `startup_from_source` only LOADS — the body runs when we `eval_in_frozen`
//! the explicit `(:my::compute)` call (the `run()` pattern from `wat_names_are_values`).
//!
//! RED at HEAD: `:wat::core::map` is an eager Rust intrinsic (`src/collection/transform.rs`) — it
//! applies `boom` to EVERY element at the `map` call, so `boom(99)` (div-by-zero) fires → eval Errs.
//! GREEN after 118.2a: `map` defers → only `boom(1)` runs → returns `1`.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

#[test]
fn lazy_core_map_does_not_force_late_elements() {
    let src = r#"
        (:wat::core::defn :my::compute [] -> :wat::core::i64
          (:wat::core::let
            [boom (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                    (:wat::core::if (:wat::core::= x 99)
                      -> :wat::core::i64
                      (:wat::core::i64::/ x 0)
                      x))
             mapped (:wat::core::map boom (:wat::core::Vector :wat::core::i64 1 2 99))]
            (:wat::core::first mapped)))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let call = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    let result = eval_in_frozen(&call, &world, &env);
    assert!(
        result.is_ok(),
        "core::map must be LAZY — mapping a fn that errors on a late element, then pulling only the \
         head, must not force the late div-by-zero; got: {:?}",
        result.err()
    );
}
