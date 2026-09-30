//! Arc 255 STONE 71 (THE WALL) — the typed twin of
//! `probe_stone255_71_template_untyped.rs`/`.wat.bad`: once a macro's OWN template names its
//! constructor's type, every use passes and runs, the same as ordinary code.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn typed_constructor_in_a_macro_template_runs() {
    match call_beside_value(file!(), ":user::use-pair-typed") {
        Ok(Value::Tuple(t)) => {
            assert_eq!(t.len(), 2, "(wat.type/Tuple :- [i64 i64] 1 2) must yield a 2-tuple");
        }
        other => panic!("a typed constructor inside a macro template must run; got {other:?}"),
    }
}
