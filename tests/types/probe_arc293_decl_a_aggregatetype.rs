//! RED probe — arc 293 decl-a: `aggregatetype` is the ONE type-reg primitive; holder DERIVED from parent root.
//!
//! The declaration unification (293 audit — the holder is a passing policy; the declaration is
//! holder-agnostic). `structtype` + `recordtype` collapse into one `(:wat::core::aggregatetype
//! :Name :Parent [fields])` whose holder is `root_holder_of(:Parent)`:
//!   :wat::core::Struct → Struct · :wat::Record → Record · :wat::holon::Record → HolonRecord
//! decl-a mints the primitive + one `parse_aggregate` + the `:wat::core::Struct` lattice node
//! (structs repoint root Value → Struct, behaviour-preserving: `Struct <: Value`). `structtype`/
//! `recordtype` stay as thin aliases routing to `parse_aggregate` (current surface unchanged).
//!
//! RED at HEAD: `:wat::core::aggregatetype` is an unknown type head AND `:wat::core::Struct` is
//! not a registered node → the fixture fails to load. GREEN after decl-a: a struct declared via
//! the unified primitive registers (holder=Struct from its `:wat::core::Struct` parent root) and
//! its codegen'd ctor + accessor work.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// A struct declared via `aggregatetype` (parent = the `:wat::core::Struct` holder root)
/// constructs via its bare ctor and reads field `a` back = 7.
#[test]
fn aggregatetype_declares_struct_via_struct_root() {
    let world = startup_beside(file!())
        .expect("startup must succeed (aggregatetype + :wat::core::Struct node resolve)");
    let ast = wat::parse_one!("(:user::da-st-a)").expect("parse da-st-a call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new()).expect("eval da-st-a");
    match tv.value_owned() {
        Value::i64(7) => {}
        other => panic!("aggregatetype struct: expected i64(7), got {:?}", other),
    }
}
