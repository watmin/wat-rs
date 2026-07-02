//! Arc 299.1 — entropic v4 conformance: Rust mints the UUID, wat measures it.
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn uuid_v4_conforms_to_v4_spec() {
    let world = startup_beside(file!()).expect("startup");
    let generated = wat_edn::new_uuid_v4();
    let call = format!("(:probe::measure \"{generated}\")");
    let ast = wat::parse_one!(&call).expect("parse");
    let verdict = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned();
    match verdict {
        Value::bool(true) => {}
        other => panic!("uuid-v4 failed conformance for {generated}: {other:?}"),
    }
}
