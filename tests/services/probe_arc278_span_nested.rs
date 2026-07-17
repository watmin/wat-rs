//! arc 278 — nested units of work via nested with-span. outer (ns "outer-ns", incr :o) wraps inner
//! (ns "inner-ns", incr :i); each closes independently and emits into its own namespace. query each
//! back: outer-ns has 1 metric, inner-ns has 1 metric. Encoded 1*10+1 = 11. Proves nesting works
//! (un-correlated); parent->child correlation is the enhancement still to build.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn nested_with_span_emits_into_each_namespace_independently() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("nested with-span raised: {e:?}"));
    assert!(matches!(got, Value::i64(11)),
        "expected outer-ns=1 metric and inner-ns=1 metric (1*10+1=11); got {got:?}");
}
