//! Stone 255.44 — span' reports the journal write.
//!
//! The store's `put` is `Success`, `Fatal`, `Constraint`, or `Transient`.
//! `Span/log` must come back as that same arm. Pre-stone every arm was
//! `LogResponse.Ok`.

use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn log_through(mode: &str) -> Value {
    let world = startup_beside(file!()).expect("the probe loads");
    let func = world
        .symbols()
        .get(":user::log-through")
        .expect(":user::log-through")
        .clone();
    apply_function(
        func,
        vec![Value::String(Arc::new(mode.to_string()))],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .unwrap_or_else(|err| panic!("log-through {mode} returned {err:?}"))
}

fn variant(v: &Value) -> &wat::value::value::EnumValue {
    match v {
        Value::Enum(ev) => {
            assert_eq!(ev.type_path, ":wat::telemetry::Span::LogResponse");
            ev
        }
        other => panic!("log must return LogResponse, got {other:?}"),
    }
}

fn aggregate_fields<'a>(v: &'a Value, class: &str) -> &'a [Value] {
    match v {
        Value::Aggregate(a) if a.class.as_ref() == class => &a.fields,
        other => panic!("expected a {class} aggregate; got {other:?}"),
    }
}

fn fault_message(err: &Value, class: &str) -> String {
    let record = aggregate_fields(err, class);
    let fault = aggregate_fields(&record[0], "wat::query::Fault");
    match &fault[0] {
        Value::String(s) => (**s).clone(),
        other => panic!("fault message must be a string; got {other:?}"),
    }
}

#[test]
fn a_successful_write_is_ok() {
    let got = log_through("ok");
    let ev = variant(&got);
    assert_eq!(ev.variant_name, "Ok");
    assert_eq!(ev.fields.len(), 0);
}

#[test]
fn a_fatal_write_is_fatal() {
    let got = log_through("fatal");
    let ev = variant(&got);
    assert_eq!(ev.variant_name, "Fatal");
    assert_eq!(fault_message(&ev.fields[0], "wat::query::Fatal"), "SPAN-SINK-FATAL");
}

#[test]
fn a_constraint_write_is_constraint() {
    let got = log_through("constraint");
    let ev = variant(&got);
    assert_eq!(ev.variant_name, "Constraint");
    assert_eq!(
        fault_message(&ev.fields[0], "wat::query::Constraint"),
        "SPAN-SINK-CONSTRAINT"
    );
}

#[test]
fn a_transient_write_is_transient() {
    let got = log_through("transient");
    let ev = variant(&got);
    assert_eq!(ev.variant_name, "Transient");
    assert_eq!(
        fault_message(&ev.fields[0], "wat::query::Transient"),
        "SPAN-SINK-TRANSIENT"
    );
}
