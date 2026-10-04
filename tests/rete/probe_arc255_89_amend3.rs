//! Stone 255.89 amend 3 — inline cond and oracle negation, both spellings.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn call(entry: &str) -> Value {
    call_beside_value(file!(), entry).unwrap_or_else(|e| panic!("{entry}: {e:?}"))
}

#[test]
fn inline_cond_keeps_the_row_above_one_hundred_in_each_spelling() {
    assert_eq!(call(":user::inline-kw"), Value::i64(1));
    assert_eq!(call(":user::inline-sym"), Value::i64(1));
}

#[test]
fn rule_negates_in_sees_a_nested_not_in_each_spelling() {
    assert_eq!(call(":user::neg-kw"), Value::i64(1));
    assert_eq!(call(":user::neg-sym"), Value::i64(1));
}
