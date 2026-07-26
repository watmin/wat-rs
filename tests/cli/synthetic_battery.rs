//! Arc 170 — the OWED guard for `wat::distribution::Battery`.
//!
//! Stone 5 (`83093431`) annihilated `wat-lru` + `wat-holon-lru`, the last
//! two things exercising the battery extension path, and deleted the only
//! compile-time proof of the shape (`crates/wat-cli/tests/wat_arc100_public_api.rs`).
//! `Battery` and `run` are **published surface** (see `src/distribution/mod.rs`'s
//! module docs) — a stated capability with zero traffic stops being a wall
//! (arc 278 R-series). This file is that traffic, restored WITHOUT depending
//! on any real extension crate.
//!
//! The deleted file's assertions were vacuous: `assert_eq!(slice.len(), 2)`
//! on a two-element literal can never fail. These tests deliberately do NOT
//! reproduce that shape. Instead, each battery function is actually CALLED
//! through the `Battery`-typed slice, and the assertions are on what calling
//! it produces — a `RustDepsBuilder` that really gained the registered type,
//! and a `wat_sources()` that really returns the baked source. If `Battery`'s
//! pair signature drifts (arg types, return types, arity), this file fails to
//! COMPILE; if the tuple ever silently stored the wrong function pointer
//! (a mis-paired `(register, wat_sources)` literal), these tests fail to PASS.
//!
//! Signatures grounded from `src/rust_deps/cache.rs`'s live `#[wat_dispatch]`
//! usage — `pub fn register(builder: &mut RustDepsBuilder)` — and
//! `src/source.rs`'s documented two-part external-crate contract —
//! `pub fn wat_sources() -> &'static [WatSource]`. Together they are exactly
//! `wat::distribution::Battery`.

use wat::distribution::Battery;
use wat::rust_deps::{RustDepsBuilder, RustTypeDecl};
use wat::WatSource;

// The two synthetic batteries' wat source, co-located `.wat` fixtures
// (`no_inlined_wat_in_tests` — a real `#[wat_dispatch]` crate's
// `wat_sources()` is "typically baked via `include_str!` from the crate's
// `wat/` directory" per `src/source.rs`'s documented contract; an inline
// Rust string literal here would be LESS faithful to the real shape, not
// more convenient, so the fixture earns no exemption).
const ALPHA_WAT: &str = include_str!("synthetic_battery__alpha.wat");
const BETA_WAT: &str = include_str!("synthetic_battery__beta.wat");

// ─── Synthetic pair "alpha" — stands in for one downstream extension crate ──

fn synthetic_alpha_register(builder: &mut RustDepsBuilder) {
    builder.register_type(RustTypeDecl {
        path: ":rust::synthetic::Alpha",
    });
}

fn synthetic_alpha_wat_sources() -> &'static [WatSource] {
    static SOURCES: [WatSource; 1] = [WatSource {
        path: "synthetic-alpha.wat",
        source: ALPHA_WAT,
    }];
    &SOURCES
}

// ─── Synthetic pair "beta" — stands in for a SECOND downstream crate ────────

fn synthetic_beta_register(builder: &mut RustDepsBuilder) {
    builder.register_type(RustTypeDecl {
        path: ":rust::synthetic::Beta",
    });
}

fn synthetic_beta_wat_sources() -> &'static [WatSource] {
    static SOURCES: [WatSource; 1] = [WatSource {
        path: "synthetic-beta.wat",
        source: BETA_WAT,
    }];
    &SOURCES
}

/// The `(register, wat_sources)` pair coerces into a one-element `&[Battery]`
/// — the minimal shape a custom CLI's `main()` builds before handing it to
/// `wat::distribution::run`. Calling BOTH halves through the slice (not just
/// storing them) proves the tuple actually holds the right function pointers
/// in the right order, not merely that a 2-tuple of function items type-checks.
#[test]
fn battery_pair_registers_its_type_and_yields_its_sources() {
    let batteries: &[Battery] = &[(synthetic_alpha_register, synthetic_alpha_wat_sources)];

    // register half: run it through a fresh builder and check the effect
    // landed — this can fail (wrong builder mutated, wrong fn called,
    // register no-op'd) in a way a length check on a literal cannot.
    let mut builder = RustDepsBuilder::new();
    for (register, _) in batteries {
        register(&mut builder);
    }
    let registry = builder.build();
    assert!(
        registry.has_type(":rust::synthetic::Alpha"),
        "synthetic_alpha_register's effect must be visible in the built registry"
    );

    // wat_sources half: call the tuple's second element and check the
    // CONTENT it returns, not just its length. Compared against the SAME
    // `include_str!` constant the static array was built from — a real
    // round-trip check, not a re-typed copy of the fixture's text (which
    // would itself be an inlined-wat literal, defeating the point).
    let sources = batteries[0].1();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].path, "synthetic-alpha.wat");
    assert_eq!(sources[0].source, ALPHA_WAT);
}

/// A multi-battery `&[Battery]` composes in slice order — the shape
/// `install_batteries` (`src/distribution/battery.rs`) actually iterates:
/// every battery's `register` runs against the SAME builder, and
/// `wat_sources()` results collect into a `Vec` a distributor's dep-source
/// installer consumes positionally. Proves two independent extension
/// crates' batteries don't clobber each other.
#[test]
fn battery_slice_composes_multiple_pairs_without_clobbering() {
    let batteries: &[Battery] = &[
        (synthetic_alpha_register, synthetic_alpha_wat_sources),
        (synthetic_beta_register, synthetic_beta_wat_sources),
    ];

    let mut builder = RustDepsBuilder::new();
    for (register, _) in batteries {
        register(&mut builder);
    }
    let registry = builder.build();
    assert!(registry.has_type(":rust::synthetic::Alpha"));
    assert!(registry.has_type(":rust::synthetic::Beta"));

    // wat_sources order mirrors battery order — the shape
    // `install_batteries` relies on when it builds `dep_sources`.
    let dep_sources: Vec<&'static [WatSource]> =
        batteries.iter().map(|(_, sources)| sources()).collect();
    assert_eq!(dep_sources.len(), 2);
    assert_eq!(dep_sources[0][0].path, "synthetic-alpha.wat");
    assert_eq!(dep_sources[1][0].path, "synthetic-beta.wat");
}
