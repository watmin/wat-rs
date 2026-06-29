//! Arc 293.2-rename — DISCONFIRMING PROBE: the record decl macros reach their FINAL names.
//!
//! The aggregate trio is `defstruct` / `defrecord` / `holon::defrecord`, all thin macros over the
//! `structtype` / `recordtype` primitives (DESIGN §, R2 fulfillment). `defstruct` already landed its
//! final name (`:wat::core::defstruct`, 293.2-parity). This strike renamed the two record macros
//! to their final heads:
//!   :wat::core::Record::def        -> :wat::core::defrecord
//!   :wat::holon::Record::def -> :wat::holon::defrecord   (a reclaimed name; hard-cut at Stone 234.6)
//!
//! The rename was SURGICAL: only the `::def` macro moved. The sibling names survive untouched —
//! `:wat::core::Record::of` (the ctor primitive), `:wat::core::Record/field-at` (the accessor), and `:wat::core::Record`
//! (the holder TYPE / lattice root). The fix-wat `rename-keyword-prefix` is boundary-aware and nothing
//! else started with `:wat::core::Record::def`, so the prefix match was exact.
//!
//! RED at the pre-rename HEAD: `:wat::core::defrecord` / `:wat::holon::defrecord` were unknown
//! declaration heads. GREEN now: they ARE the record decl macros; the old `::def` heads throw the
//! retirement remedy.

use wat::freeze::startup_from_file;

/// The core record macro reaches `:wat::core::defrecord` (peer to `:wat::core::defstruct`).
#[test]
fn core_defrecord_is_the_record_decl_head() {
    // RED AT HEAD: :wat::core::defrecord is an unknown declaration head.
    // GREEN: it registers :geo::Pt (a core record) with its positional ctor + accessors.
    let world = startup_from_file("tests/types/probe_arc293_defrecord_rename_core.wat");
    assert!(
        world.is_ok(),
        ":wat::core::defrecord should be the core record decl head; got: {:?}",
        world.err()
    );
}

/// The holon record macro reaches the reclaimed `:wat::holon::defrecord`.
#[test]
fn holon_defrecord_is_the_holon_record_decl_head() {
    // RED AT HEAD: :wat::holon::defrecord is an unknown declaration head.
    // GREEN: it registers :geo::HPt (a holon record) — widens to :wat::holon::Record.
    let world = startup_from_file("tests/types/probe_arc293_defrecord_rename_holon.wat");
    assert!(
        world.is_ok(),
        ":wat::holon::defrecord should be the holon record decl head; got: {:?}",
        world.err()
    );
}
