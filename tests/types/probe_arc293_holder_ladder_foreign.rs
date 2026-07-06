//! RED probe — arc 293 K1b: the foreign half of the nature ladder (the (b') honesty guarantee).
//!
//! A foreign type satisfies a surface via its `extend-type` subtype edge (`assignable` arms at
//! check.rs:14633/14641). That edge must ALSO honor a nature bound: the foreign's DERIVED nature
//! (`is_holon_or_vector` -> HolonRecord, `is_portable_type` -> Record, else Struct) must clear the
//! surface's floor via `rank() >=`. A non-holon foreign (String, Record-capable) must NOT satisfy a
//! `:nature :HolonRecord` surface.
//!
//! RED at HEAD: the edge is nature-EXEMPT (option (b)), so the String wrongly satisfies and the world
//! type-checks (startup is Ok). GREEN after K1b: the derived nature (Record) < the floor (HolonRecord),
//! so the world fails to type-check (startup is Err).

use wat::freeze::startup_beside;

#[test]
fn foreign_nature_is_checked_a_nonholon_cannot_satisfy_a_holon_floor_surface() {
    let result = startup_beside(file!());
    assert!(
        result.is_err(),
        "a non-holon foreign type (String, Record-capable) must NOT satisfy a :nature :HolonRecord \
         surface — the extend-type edge must honor the nature ladder; got Ok (nature-exempt)"
    );
}
