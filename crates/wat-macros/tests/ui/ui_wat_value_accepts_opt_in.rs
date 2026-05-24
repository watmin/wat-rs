// compile-pass fixture: opt-in escape hatch with non-empty reason compiles.
//
// Contract 4 from sub-DESIGN DESIGN-STONE-233.2.l.md.
// Verifies that a variant with Box<Self> field AND a per-variant
// #[wat_value(allow_wrapping = "reason")] opt-in compiles cleanly.
//
// The reason string documents WHY the structural exception is justified.

use wat_macros::wat_value;

#[wat_value]
pub enum DocumentedWrapValue {
    Leaf(i64),
    // Opt-in: the reason string is mandatory and non-empty.
    #[wat_value(allow_wrapping = "demonstration only — no real use case; see arc 233 doctrine")]
    Wrap { inner: Box<DocumentedWrapValue> },
}

fn main() {
    let v = DocumentedWrapValue::Leaf(42);
    assert!(matches!(v, DocumentedWrapValue::Leaf(42)));

    let w = DocumentedWrapValue::Wrap {
        inner: Box::new(DocumentedWrapValue::Leaf(1)),
    };
    assert!(matches!(w, DocumentedWrapValue::Wrap { .. }));
}
