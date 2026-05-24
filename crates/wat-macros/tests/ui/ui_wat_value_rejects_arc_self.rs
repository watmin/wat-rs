// compile-fail fixture: Arc<Self> field should be rejected by #[wat_value].
//
// Contract 2 from sub-DESIGN DESIGN-STONE-233.2.l.md.
// Verifies the trap-door class detection catches Arc<EnumName> field.

use std::sync::Arc;
use wat_macros::wat_value;

#[wat_value]
pub enum BadArcValue {
    Leaf(i64),
    Wrap(Arc<BadArcValue>),
}

fn main() {}
