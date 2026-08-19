//! B1 regression probe — `struct->form` + `eval-ast!` roundtrip (NON-CONCURRENT).
//!
//! Arc 293.R2 collapsed the three struct representations into `Value::Aggregate`;
//! R2.3 dropped `:T/new` in favour of the bare `:T` constructor.
//! `eval_struct_to_form` (runtime.rs:9511) was NOT updated — it still formatted
//! the WatAST constructor keyword as `":{}/new"`. After the B1 fix it emits bare `":{}"`,
//! and `eval-ast!` on the lifted form can reconstruct the struct.
//!
//! RED before fix: `eval-ast!` returns `Value::Result(Err(...))` — `UnknownFunction :probe::Pair/new`.
//! GREEN after fix: `Value::Result(Ok(Value::Aggregate { class: "probe::Pair", fields: [i64(7), i64(9)] }))`.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn struct_to_form_eval_ast_roundtrip() {
    let got = call_beside_value(file!(), ":probe::roundtrip")
        .expect("(:probe::roundtrip) must not throw at the eval level");

    let inner = match got {
        Value::Result(r) => r,
        other => panic!("expected Value::Result from (:probe::roundtrip); got {other:?}"),
    };

    match inner.as_ref() {
        Ok(Value::Aggregate(agg)) => {
            assert_eq!(
                agg.class.as_ref(), "probe::Pair",
                "reconstructed struct class mismatch: expected 'probe::Pair', got {:?}",
                agg.class
            );
            match &agg.fields[0] {
                Value::i64(n) => assert_eq!(*n, 7, "field `a` must round-trip to 7; got {n}"),
                other => panic!("field `a` must be Value::i64(7); got {other:?}"),
            }
        }
        Ok(other) => panic!(
            "B1: eval-ast! Ok but NOT an Aggregate — expected reconstructed probe::Pair; got {other:?}"
        ),
        Err(e) => panic!(
            "B1 REGRESSION: eval-ast! returned Err — struct->form still emits dead ':T/new' ctor \
             (runtime.rs format!(\":{{}}/new\") not updated to format!(\":{{}}\")). Err: {e:?}"
        ),
    }
}
