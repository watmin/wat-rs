//! RED probe — arc 293 K1b: the foreign half of the holder ladder (the (b') honesty guarantee).
//!
//! A foreign type satisfies a surface via its `extend-type` subtype edge (`assignable` arms at
//! check.rs:14633/14641). That edge must ALSO honor a holder bound: the foreign's DERIVED holder
//! (`is_holon_or_vector` -> HolonRecord, `is_portable_type` -> Record, else Struct) must clear the
//! surface's floor via `rank() >=`. A non-holon foreign (String, Record-capable) must NOT satisfy a
//! `:holder :HolonRecord` surface.
//!
//! RED at HEAD: the edge is holder-EXEMPT (option (b)), so the String wrongly satisfies and the world
//! type-checks (startup is Ok). GREEN after K1b: the derived holder (Record) < the floor (HolonRecord),
//! so the world fails to type-check (startup is Err).

use wat::freeze::startup_beside;

#[test]
fn foreign_holder_is_checked_a_nonholon_cannot_satisfy_a_holon_floor_surface() {
    let result = startup_beside(file!());
    assert!(
        result.is_err(),
        "a non-holon foreign type (String, Record-capable) must NOT satisfy a :holder :HolonRecord \
         surface — the extend-type edge must honor the holder ladder; got Ok (holder-exempt)"
    );
}
