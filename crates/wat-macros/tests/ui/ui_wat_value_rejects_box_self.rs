// compile-fail fixture: Box<Self> field should be rejected by #[wat_value].
//
// Contract 1 from sub-DESIGN DESIGN-STONE-233.2.l.md.
// Verifies the trap-door class detection catches Box<EnumName> field.

use wat_macros::wat_value;

#[wat_value]
pub enum BadValue {
    Leaf(i64),
    Wrap { inner: Box<BadValue> },
}

fn main() {}
