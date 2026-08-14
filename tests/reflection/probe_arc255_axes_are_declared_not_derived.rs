//! probe_arc255_axes_are_declared_not_derived — the intrinsic doc contract's two purity axes are
//! DECLARED and parsed, not inferred from a namespace.
//!
//! ## Why this exists
//!
//! `src/intrinsic/mod.rs`'s module header says:
//!
//!     "purity / determinism → DERIVED at the reflection site … not stored on the entry"
//!
//! **That sentence is STALE, and it misled a reader (the orchestrator, 2026-08-02) into telling
//! the builder that arc 255's registry only guesses at purity.** The code disagrees with its own
//! comment, and the code is the writer:
//!
//!   - `IntrinsicEntry.purity: wat_doc::Purity` / `.determinism: wat_doc::Determinism` are STORED
//!     fields, doc-commented "Declared purity — from `@Purity <Variant>` in the doc."
//!   - `@Purity Pure` / `@Determinism Deterministic` are LIVE tags on real handlers
//!     (`src/intrinsic/bytes.rs:36-38`).
//!   - `pure_declared_matches_is_effectful_op` (cfg(test), `intrinsic/mod.rs:596`) already asserts
//!     the declared value agrees with `is_effectful_op` for every registered entry.
//!
//! Derivation (`derive_pure_deterministic`, `runtime.rs:24371`) is the fallback for verbs that are
//! NOT enrolled — not the model for ones that are. This file pins that, so the next reader who
//! meets the stale header has a green test contradicting it.
//! ([[feedback_ground_a_fields_liveness_by_its_writer_not_its_comment]] — the lesson, re-lived.)
//!
//! ## Two facts recorded while measuring, for whoever picks up arc 255
//!
//! Neither is acted on here; both are filed in
//! `docs/arc/2026/06/255-builtin-registry/NOTE-purity-is-definition-time-queryable-metadata.md`.
//!
//!   1. **The doc contract cannot carry a third axis.** `wat_doc::parse`'s recognized-tag list
//!      (`crates/wat-doc/src/lib.rs:321-322`) is closed — `@added @arg @ret @example
//!      @example-norun @deprecated @see @Purity @Determinism @Category @yields`. A `@Total` is
//!      refused as `UnknownDirective` (verified by run, 2026-08-02). **Totality is the one axis a
//!      namespace prefix cannot derive** — `:wat::core::i64::+` is pure ∧ deterministic ∧ NOT
//!      total — which is why the rete fence carries its own `total` column (#52) and will keep
//!      carrying it until 255 arrives. Nothing needs `@Total` today.
//!   2. **`wat_doc::Category` has no arithmetic variant** — the closed set is
//!      `Encoding | Reflection | ControlFlow | Binding | Clock | Arithmetic`
//!      (append-only; see `Category::variants()`). Whenever 255 enrols
//!      the arithmetic families it grows TWO closed sets, not one.
//!
//! ## Reading it
//!
//!     cargo test --release --test reflection -- axes_are_declared
//!
//! All three must be GREEN. Control 3 is what makes 1 and 2 trustworthy: it proves an unknown
//! `@`-directive is REFUSED rather than silently dropped, so a doc that parses is a doc whose
//! every tag was understood.

use wat_doc::{DocError, Determinism, Purity};

/// The shared doc body. One place, so any case differs by exactly the line under test.
///
/// Grammar copied from a LIVE handler (`src/intrinsic/bytes.rs:32-43`), not invented:
/// `@arg <name> <type> <desc>`. A separator in the type position is a `MalformedDirective` — a
/// first draft wrote `@arg a — the left operand` (the form in `#[wat_intrinsic]`'s own stale doc
/// example) and every control went red on it. The controls catching the probe's own bug rather
/// than reporting a substrate finding is the instrument working
/// ([[feedback_the_instrument_must_not_supply_the_result]]).
///
/// `@Category Encoding` is deliberately wrong-but-valid for an arithmetic example — see fact 2 in
/// the module header. The category is irrelevant to what this file measures.
fn doc_with(extra_tag_lines: &str) -> String {
    format!(
        "Add two i64s, yielding a declared fallback on overflow.\n\
         \n\
         @added         1.0.0\n\
         @Purity        Pure\n\
         @Determinism   Deterministic\n\
         @Category      Encoding\n\
         {extra_tag_lines}\
         @arg     a :wat::core::i64 the left operand\n\
         @arg     b :wat::core::i64 the right operand\n\
         @ret     :wat::core::i64 the sum, or the declared fallback on overflow\n\
         @example (:probe::add 1 2) #=> 3\n"
    )
}

/// CONTROL 1 — the shared body parses clean. Non-vacuity floor: if this fails, every other
/// verdict in this file is about a malformed doc rather than about the contract.
#[test]
fn control_baseline_doc_parses() {
    let doc = wat_doc::parse(&doc_with("")).expect("baseline doc must parse — the probe's floor");
    assert_eq!(doc.added, "1.0.0");
    assert_eq!(doc.args.len(), 2, "both @arg directives survive the parse");
}

/// ★ THE CLAIM — both purity axes are read OFF THE DOC. Refutes the stale module header.
#[test]
fn axes_are_declared_not_derived() {
    let doc = wat_doc::parse(&doc_with("")).expect("baseline doc must parse");
    assert_eq!(doc.purity, Purity::Pure, "@Purity is parsed from the doc, not inferred");
    assert_eq!(
        doc.determinism,
        Determinism::Deterministic,
        "@Determinism is parsed from the doc, not inferred"
    );
}

/// CONTROL 2 — an unrecognized `@`-directive is REFUSED, not ignored. This is what makes a
/// successful parse meaningful: every tag in a doc that parses was understood. A silently-dropped
/// tag would be the worse failure — the entry would carry a default and nobody would learn the
/// declaration never landed.
#[test]
fn unknown_directive_is_rejected_not_ignored() {
    let err = wat_doc::parse(&doc_with("@NotARealTag Whatever\n"))
        .expect_err("an unrecognized @-directive must be refused");
    match err {
        DocError::UnknownDirective { tag } => assert_eq!(tag, "@NotARealTag"),
        other => panic!("expected UnknownDirective, got {other:?}"),
    }
}
