//! RED probe — arc 293 decl-a: `aggregatetype` is the ONE type-reg primitive; nature DERIVED from parent root.
//!
//! The declaration unification (293 audit — the nature is a passing policy; the declaration is
//! nature-agnostic). `structtype` + `recordtype` collapse into one `(:wat::core::aggregatetype
//! :Name :Parent [fields])` whose nature is `root_nature_of(:Parent)`:
//!   :wat::core::Struct → Struct · :wat::core::Record → Record · :wat::holon::Record → HolonRecord
//! decl-a mints the primitive + one `parse_aggregate` + the `:wat::core::Struct` lattice node
//! (structs repoint root Value → Struct, behaviour-preserving: `Struct <: Value`). `structtype`/
//! `recordtype` stay as thin aliases routing to `parse_aggregate` (current surface unchanged).
//!
//! RED at HEAD: `:wat::core::aggregatetype` is an unknown type head AND `:wat::core::Struct` is
//! not a registered node → the fixture fails to load. GREEN after decl-a: a struct declared via
//! the unified primitive registers (nature=Struct from its `:wat::core::Struct` parent root) and
//! its codegen'd ctor + accessor work.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// A struct declared via `aggregatetype` (parent = the `:wat::core::Struct` nature root)
/// constructs via its bare ctor and reads field `a` back = 7.
#[test]
fn aggregatetype_declares_struct_via_struct_root() {
    let got = call_beside(file!(), ":user::da-st-a").expect("eval da-st-a");
    match got {
        Value::i64(7) => {}
        other => panic!("aggregatetype struct: expected i64(7), got {:?}", other),
    }
}
