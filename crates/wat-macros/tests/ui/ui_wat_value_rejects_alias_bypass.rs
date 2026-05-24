// compile-pass fixture: DOCUMENTED LIMITATION — type alias bypass.
//
// Contract 5 from sub-DESIGN DESIGN-STONE-233.2.l.md, Decision 1.
//
// This fixture demonstrates the known limitation of #[wat_value]'s syntactic
// scan: a type alias that resolves to a forbidden type (e.g., `Box<SomeEnum>`)
// BYPASSES the seal because the macro sees only the alias name, not its
// expansion.
//
// Per Decision 1 of DESIGN-STONE-233.2.l.md: semantic resolution would require
// rustc internals out of scope. The opt-in escape hatch covers the legitimate-
// exception case. Authors who use aliases to wrap Self ARE bypassing the seal
// intentionally — they must document the intent in code comments.
//
// The recommended pattern if you genuinely need a wrapped Self via alias:
//   - Add `#[wat_value(allow_wrapping = "reason")]` to the variant explicitly,
//     making the intent part of the compile-time record.
//   - Or don't use an alias — use the canonical form so #[wat_value] can see it.
//
// This fixture compiles (bypass succeeds). The BRIEF and DESIGN document this
// as an accepted limitation.

use wat_macros::wat_value;

// An alias that resolves to Box<AliasedValue>. The macro sees `BoxedAliased`,
// not `Box<AliasedValue>`, so it does NOT trigger the wrapping-variant error.
type BoxedAliased = Box<AliasedValue>;

#[wat_value]
pub enum AliasedValue {
    Leaf(i64),
    // NOTE: This bypasses #[wat_value]'s seal via type alias indirection.
    // This is the KNOWN LIMITATION per Decision 1 of DESIGN-STONE-233.2.l.md.
    // In production code this would require explicit opt-in:
    //   #[wat_value(allow_wrapping = "reason")]
    // to make the intent visible. Without the opt-in, the alias bypass is
    // a disciplinary gap — documented, not structural.
    Wrap { inner: BoxedAliased },
}

fn main() {
    let v = AliasedValue::Leaf(0);
    assert!(matches!(v, AliasedValue::Leaf(0)));
}
