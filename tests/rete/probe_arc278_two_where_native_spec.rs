//! Two standalone `:where` (mid-chain and trailing). Native `fire-rules`
//! must derive the same bag as `fire-rules-spec` (Oslo only). Spec is the
//! oracle; native is the user path. A disagreement is a rete flaw.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

#[test]
fn two_where_native_matches_spec_oslo_only() {
    let native = call(":user::native-count");
    let spec = call(":user::spec-count");
    assert_eq!(
        spec,
        Value::i64(1),
        "spec must derive Oslo only; got {spec:?}"
    );
    assert_eq!(
        native, spec,
        "native fire-rules must match fire-rules-spec on two standalone :where \
         (mid-chain + trailing); native={native:?} spec={spec:?}"
    );
}
