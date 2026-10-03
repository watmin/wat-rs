//! Stone 255.87 group A #1 — symbol declaration names, and a surface in an
//! extend-type binder tail.
//!
//! `probe/hold` returns i64 42 only when each symbol-spelled declaration is
//! present under its canonical keyword and both spellings of `inc` / `mac`
//! return the same value. The keyword `defservice` is the floor's shape: its
//! expansion names `wat.capability/Capability` in that tail.

use wat::freeze::call_beside_value;
use wat::Value;

#[test]
fn symbol_surface_in_extend_type_is_the_canonical_identity() {
    let v = call_beside_value(file!(), ":probe::hold").expect("hold");
    // Bits, low to high: Rec En Surf Cap Box kwRec KwCap Api svcState
    // inc kw-inc mac kw-mac. 8191 is every bit set.
    assert_eq!(v, Value::i64(8191));
}
