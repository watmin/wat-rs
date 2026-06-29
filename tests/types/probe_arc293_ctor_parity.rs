//! Arc 293 — DISCONFIRMING PROBE for CONSTRUCTION PARITY (drop `/new`; the type name IS the ctor).
//!
//! Builder's call (2026-06-26, `293/NOTE-base-struct-horizon.md`): *"i want parity in construction… using
//! structs vs core-records vs holon-records should be identical… dropping /new feels like the better thing"* —
//! and newtypes fold in (*"the name is the ctor just like records"*). So EVERY type-name is its own
//! constructor — struct, core-record, holon-record, newtype — all via the bare `:T`:
//!   (:geo::SPt 1 2)  ==  (:geo::Circle "red" 2.0)  ==  (:my::Amount 42)
//! `:T/new` is annihilated.
//!
//! RED at HEAD: the struct + newtype ctors live at `:T/new` (`register_struct_methods` / newtype codegen,
//! `runtime.rs:923/1175`); the bare `:T` does not resolve as a constructor function. Records already
//! construct via the bare `:T` (defrecord emits a `defn` at the fqdn) — this strike gives structs + newtypes
//! the same surface. GREEN when the ctor registers at `:T` and the `:T/new` call sites migrate.

use wat::freeze::startup_from_file;

/// A STRUCT constructs via its bare type name — `(:geo::SPt 3 4)`, parity with a record's `(:geo::Circle …)`.
#[test]
fn struct_constructs_via_bare_type_name() {
    // GREEN TARGET: (:geo::SPt 3 4) constructs the struct (the bare type name is the ctor).
    // RED AT HEAD: the ctor is :geo::SPt/new; bare :geo::SPt is unresolved as a function.
    let world = startup_from_file("tests/types/probe_arc293_ctor_parity_struct.wat");
    assert!(
        world.is_ok(),
        "a struct should construct via its bare type name :geo::SPt (parity with records); got: {:?}",
        world.err()
    );
}

/// A NEWTYPE constructs via its bare type name — `(:my::Amount 42)`, no `/new`.
#[test]
fn newtype_constructs_via_bare_type_name() {
    // GREEN TARGET: (:my::Amount 42) wraps the value (the bare type name is the ctor).
    // RED AT HEAD: the ctor is :my::Amount/new; bare :my::Amount is unresolved as a function.
    let world = startup_from_file("tests/types/probe_arc293_ctor_parity_newtype.wat");
    assert!(
        world.is_ok(),
        "a newtype should construct via its bare type name :my::Amount (parity); got: {:?}",
        world.err()
    );
}
