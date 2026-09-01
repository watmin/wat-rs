//! Queue depth: `stats` reports pending (visible) and in-flight (received, not
//! acked). Drives `:user::depth` in `wat-scripts/queue/sqs.wat`.

use wat::freeze::startup_from_file;

#[test]
fn queue_depth_counters_are_accurate() {
    let world = startup_from_file("wat-scripts/queue/sqs.wat")
        .expect("startup should succeed (queue + mem-store' baked)");
    let func = world
        .symbols()
        .get(":user::depth")
        .unwrap_or_else(|| panic!(":user::depth not registered"))
        .clone();
    let stored = match wat::runtime::apply_function(
        func,
        vec![],
        world.symbols(),
        wat::rust_caller_span!(),
    ) {
        Ok(wat::runtime::Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::depth returned non-String: {other:?}"),
        Err(e) => panic!("depth gate raised: {e:?}"),
    };
    assert_eq!(
        stored, "send=p=3,f=0;recv=p=1,f=2;ack=p=1,f=1",
        "send 3 → pending=3 in-flight=0; receive 2 → pending=1 in-flight=2; ack 1 → pending=1 in-flight=1. got: {stored}"
    );
}
