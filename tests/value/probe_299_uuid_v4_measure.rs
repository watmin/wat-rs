//! Arc 299.1 — entropic v4 conformance: Rust mints the UUID, wat measures it.
use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn uuid_v4_conforms_to_v4_spec() {
    let world = startup_beside(file!()).expect("startup");
    let generated = wat_edn::new_uuid_v4();
    // just-eval (rubric): fetch the fixture fn and `apply_function` it with a Rust-constructed
    // `Value::String` arg — no inline wat driver built via `format!`.
    let func = world
        .symbols()
        .get(":probe::measure")
        .expect("no :probe::measure in fixture")
        .clone();
    let verdict = apply_function(
        func,
        vec![Value::String(Arc::new(generated.to_string()))],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("eval");
    match verdict {
        Value::bool(true) => {}
        other => panic!("uuid-v4 failed conformance for {generated}: {other:?}"),
    }
}
