//! Long polling in wat-queue. Drives `:user::long-poll` in
//! `wat-scripts/queue/sqs.wat`. Existing `:user::compute` lifecycle is
//! unedited (STOP-2: wait-ns = 0 is today's path).

use wat::freeze::startup_from_file;

#[test]
fn queue_long_poll_gates() {
    let world = startup_from_file("wat-scripts/queue/sqs.wat")
        .expect("startup should succeed (queue + mem-store' baked)");
    let func = world
        .symbols()
        .get(":user::long-poll")
        .unwrap_or_else(|| panic!(":user::long-poll not registered"))
        .clone();
    let stored = match wat::runtime::apply_function(
        func,
        vec![],
        world.symbols(),
        wat::rust_caller_span!(),
    ) {
        Ok(wat::runtime::Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::long-poll returned non-String: {other:?}"),
        Err(e) => panic!("long-poll gates raised: {e:?}"),
    };
    assert_eq!(
        stored,
        "wakes=got=hello;hidden=yes;timeout=empty=yes;serving=yes;fifo=a=first;c=second;fewer=n=3;calls=2;idle=ticks=0",
        "long-poll gates: send-wakes with re-put, timeout empty, FIFO, fewer receives than a spin, idle ticks=0. got: {stored}"
    );
}
