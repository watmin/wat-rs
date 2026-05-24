// compile-fail fixture: Self field directly should be rejected by #[wat_value].
//
// Contract 3 from sub-DESIGN DESIGN-STONE-233.2.l.md.
// Verifies the trap-door class detection catches a direct Self/EnumName field
// (unboxed recursive reference).

use wat_macros::wat_value;

#[wat_value]
pub enum DirectSelfValue {
    Leaf(i64),
    // A direct Self reference. In real Rust this would not compile anyway
    // (infinite size), but #[wat_value] catches it FIRST with a teaching
    // diagnostic.
    Wrap { inner: Box<Self> },
}

fn main() {}
