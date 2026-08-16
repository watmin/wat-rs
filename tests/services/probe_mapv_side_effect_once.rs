//! mapv / thread-kwargs increment exactly once per element.
//!
//! cargo nextest run -p wat -E 'test(/probe_mapv_side_effect_once/)' --test-threads=1

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn call(world: &wat::freeze::FrozenWorld, name: &str) -> Value {
    let ast = WatAST::List(
        vec![WatAST::Keyword(name.into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("{name} raised: {e:?}"))
        .value_owned()
}

#[test]
fn mapv_of_one_element_increments_once() {
    let world =
        startup_from_file("tests/services/probe_mapv_side_effect_once.wat").expect("startup");
    assert_eq!(
        call(&world, ":probe::run-mapv"),
        Value::i64(1),
        "mapv must apply f once per element (empty?/first/rest sharing one realize)"
    );
}

#[test]
fn thread_kwargs_map_increments_once_per_item() {
    let world =
        startup_from_file("tests/services/probe_mapv_side_effect_once.wat").expect("startup");
    assert_eq!(
        call(&world, ":probe::run-thread-kwargs"),
        Value::i64(4),
        "thread kwargs map of 4 items with 1 runner must increment 4 times, not 6"
    );
}
