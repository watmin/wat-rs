//! Arc 293.2-parity — DISCONFIRMING PROBE for the `structtype` primitive.
//!
//! The parity move: `defstruct` becomes a wat MACRO over a new low-level `:wat::core::structtype`
//! type-registration primitive — exactly mirroring how `:wat::Record::def` (a macro) sits over
//! `:wat::core::recordtype` (the primitive). This makes `defstruct` and `defrecord` SYMMETRIC at the
//! macro-over-primitive level, so `/from-map` (a companion macro) can later be emitted by BOTH uniformly.
//!
//! This probe targets the genuinely-new surface: `:wat::core::structtype` as a directly-usable type-reg
//! primitive. It registers a struct type (TypeDef::Struct) just like `defstruct` does today; the existing
//! Rust method-gen (`register_struct_methods`) then synthesizes its `:T/new` ctor + `:T/<field>` accessors,
//! UNCHANGED. (`defstruct`-as-a-macro emitting `structtype` is verified behavior-preserving by the existing
//! defstruct suite staying green — the SET-diff — not by this probe.)
//!
//! RED at HEAD: `:wat::core::structtype` is an unknown declaration head. GREEN when 293.2-parity lands:
//! `structtype` registers the struct, and `defstruct` is a thin macro emitting it.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn structtype_primitive_registers_a_struct() {
    let src = r#"
        (:wat::core::structtype :my::Point
          [x <- :wat::core::i64  y <- :wat::core::i64])

        (:wat::core::defn :user::main [] -> :wat::core::i64
          (:wat::core::i64::+
            (:my::Point/x (:my::Point/new 3 4))
            (:my::Point/y (:my::Point/new 3 4))))
    "#;
    // GREEN TARGET: structtype registers :my::Point (a struct); register_struct_methods synthesizes
    // :my::Point/new + :my::Point/x + :my::Point/y exactly as for defstruct today. startup type-checks.
    // RED AT HEAD: :wat::core::structtype is an unknown declaration head.
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        ":wat::core::structtype should register a struct with /new + field accessors; got: {:?}",
        world.err()
    );
}
