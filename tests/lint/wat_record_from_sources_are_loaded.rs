//! Arc 296 — every `wat_record_from!` source file must stay in the stdlib LOAD SET.
//!
//! ## The property, and why it needs a gate at all
//!
//! `wat_record_from!` reads a `.wat` declaration at BUILD time with a hand-rolled AST walk and
//! emits the `register_builtin` row. The substrate's own field walker
//! (`parse_aggregate_fields_with_splices`) reads the SAME form at LOAD time. Two readers, one
//! text — and they must agree, or the emitted registration describes a different type than the
//! declaration it claims to come from.
//!
//! Today that agreement is enforced for free: the corpus re-declares each type, and a
//! re-declaration only no-ops via arc 054's `Existing::Equivalent` arm if it is byte-equivalent
//! to the registration. Disagree, and the stdlib refuses to load outright.
//!
//! ## ⛔ WHY THIS IS NOT THE OBVIOUS TEST — the obvious one is VACUOUS
//!
//! The natural gate is "walk the form both ways and assert the two `AggregateDef`s are equal".
//! **That test can never fail.** The loader's `AggregateDef` only exists inside a frozen world;
//! if the two walks disagreed, freezing would raise `DuplicateType` and no world would build — so
//! any test that gets far enough to compare has already proven the thing it was going to assert.
//! A test whose precondition is its own conclusion is a green that proves nothing
//! ([[feedback_a_green_test_can_prove_nothing]]).
//!
//! So this gate gaurds the *other* half: not that the two readers agree — startup already forces
//! that — but that the arrangement which forces it **still exists**.
//!
//! ## What can actually go wrong
//!
//! `wat/kernel/diagnostics.wat` holds nothing but declarations that Rust already registers, so
//! its runtime load is pure no-op work. That makes it a plausible future deletion from
//! `STDLIB_FILES` on efficiency grounds — and dropping it would **silently retire the only check
//! that the macro's walk matches the substrate's**. Nothing would go red; the differential would
//! just stop happening.
//!
//! `wat/core.wat` and `wat/holon.wat` are full stdlib files and will always be re-read, so 5 of
//! the 13 are covered regardless. The 7 in `diagnostics.wat` are the exposed ones, and they are
//! why this file exists.
//!
//! Text-scan over both sources, in the style of this directory's other lints — the point is to
//! compare two *declarations* (`types.rs`'s macro calls and `stdlib.rs`'s load list), and reading
//! them as text is how a gate sees what the compiler has already collapsed.

/// Every path named by a `wat_record_from!` invocation in `src/types.rs`.
fn macro_source_paths() -> Vec<String> {
    const TYPES_RS: &str = include_str!("../../src/types.rs");
    let mut out = Vec::new();
    for (i, _) in TYPES_RS.match_indices("wat_record_from!(") {
        let rest = &TYPES_RS[i..];
        // `wat_record_from!(env, "wat/core.wat", ":wat::core::Span")` — the FIRST string literal
        // after the macro name is the source path.
        let Some(open) = rest.find('"') else { continue };
        let after = &rest[open + 1..];
        let Some(close) = after.find('"') else { continue };
        out.push(after[..close].to_string());
    }
    out
}

/// Every path in `STDLIB_FILES`, read from the load list's own text.
fn stdlib_paths() -> Vec<String> {
    const STDLIB_RS: &str = include_str!("../../src/load/stdlib.rs");
    let mut out = Vec::new();
    for (i, _) in STDLIB_RS.match_indices("path: \"") {
        let after = &STDLIB_RS[i + "path: \"".len()..];
        let Some(close) = after.find('"') else { continue };
        out.push(after[..close].to_string());
    }
    out
}

#[test]
fn every_wat_record_from_source_is_in_the_stdlib_load_set() {
    let sources = macro_source_paths();
    let loaded = stdlib_paths();

    // Non-vacuity: if the scan finds nothing, this gate is asserting over an empty set and would
    // pass no matter what. 13 types are converted as of arc 296 step 2b.
    assert!(
        sources.len() >= 13,
        "the scan found only {} `wat_record_from!` source path(s) — expected at least the 13 \
         converted in arc 296 step 2b. Either the scan broke or the macro calls moved; either way \
         this gate is measuring nothing until it is fixed.\nfound: {sources:?}",
        sources.len()
    );
    assert!(
        !loaded.is_empty(),
        "the scan found no `path:` entries in STDLIB_FILES — the gate cannot compare against an \
         empty load set"
    );

    let missing: Vec<&String> = sources.iter().filter(|s| !loaded.contains(s)).collect();
    assert!(
        missing.is_empty(),
        "a `wat_record_from!` source file left the stdlib load set: {missing:?}\n\n\
         That file is read at BUILD time by the macro's own AST walk, and at LOAD time by the \
         substrate's field walker. The load-time read is what proves the two agree — a divergence \
         makes the corpus re-declaration stop hitting `Existing::Equivalent` and the stdlib \
         refuses to load. Drop the file from STDLIB_FILES and that proof silently stops happening: \
         the macro could emit a registration that no longer matches its own declaration, and \
         nothing would say so.\n\n\
         If the removal is deliberate, the differential needs a replacement BEFORE it goes — not \
         after."
    );
}
