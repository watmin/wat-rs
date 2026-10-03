//! Stone 255.87 group A #3 — a symbol type is a fact-bind, so compile does not
//! take `first` of that atom on the accumulate branch.
//!
//! Bits, low to high: symbol fact-bind, symbol not accumulate, keyword
//! fact-bind, keyword not accumulate, field `:id` not a fact-bind, field is
//! accumulate, real accumulator not a fact-bind, real accumulator is
//! accumulate, keyword fact pattern not accumulate, the three rules compile.
//! 1023 is every bit set.

use wat::freeze::call_beside_value;
use wat::Value;

#[test]
fn symbol_type_in_a_fact_bind_is_not_an_accumulator() {
    let v = call_beside_value(file!(), ":probe::hold").expect("hold");
    assert_eq!(v, Value::i64(1023));
}
