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
//!   - `declared_purity_vs_effectful_by_prefix_census` (cfg(test), `intrinsic/mod.rs`) records
//!     every registered row where the declared value and the `effectful_by_prefix` namespace
//!     guess disagree. It was `pure_declared_matches_is_effectful_op` and asserted a
//!     biconditional until arc 255.1c site 3; see below.
//!
//! ⊘ UPDATED 2026-08-19 (arc 255.1c-kernel-ambient-ii). The sentence that used to sit here —
//! "Derivation (`derive_pure_deterministic`) is the fallback for verbs that are NOT enrolled" —
//! described a function that **no longer exists**. The builder ruled the registry is the
//! authority for a form's properties, so `is_effectful_op` now consults it first and falls back
//! to a named `effectful_by_prefix` guess only where the registry is silent;
//! `derive_pure_deterministic` lost its last caller and was deleted. Its `(pure, deterministic)`
//! two-bool shape could not have survived arc 299.3 regardless — that stone splits `Purity` into
//! `Pure | Effectful | Entropic`, and a bool cannot carry three states.
//!
//! **This file's thesis is UNCHANGED and still holds:** the axes are DECLARED, not derived.
//! What changed is that the fallback is now named as the guess it always was.
//! ([[feedback_ground_a_fields_liveness_by_its_writer_not_its_comment]] — the lesson, re-lived.)
//!
//! ## Two facts recorded while measuring, for whoever picks up arc 255
//!
//! Neither is acted on here; both are filed in
//! `docs/arc/2026/06/255-builtin-registry/NOTE-purity-is-definition-time-queryable-metadata.md`.
//!
//!   1. ⛔ **SUPERSEDED 2026-08-30 — 255 ARRIVED.** This fact used to read: *"The doc contract
//!      cannot carry a third axis … a `@Total` is refused as `UnknownDirective` (verified by run,
//!      2026-08-02) … the rete fence … will keep carrying it until 255 arrives. Nothing needs
//!      `@Total` today."* Every clause of that is now false. Stone total-T1 minted
//!      `:wat::runtime::Totality` in `wat/runtime-meta.wat`; T2 made `@Total` a recognized
//!      directive; T2b carried it into `IntrinsicEntry`; **T3 made it REQUIRED** — and this
//!      probe's own baseline went RED on `MissingTotality` the moment it did, which is how the
//!      staleness surfaced. The observation the fact rested on remains exactly right and is why
//!      the axis exists: **totality is the one axis a namespace prefix cannot derive** —
//!      `:wat::i64::+` is pure ∧ deterministic ∧ NOT total. What changed is that it now has a
//!      home. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
//!
//!      ⊘ UPDATED 2026-08-30 (arc 255 Stone expand-T3) — `@ExpandTime` went through the identical
//!      arc one stone later: minted at expand-T1, recognized (OPTIONAL) at expand-T2, **REQUIRED**
//!      here — and this probe's baseline went RED on `MissingExpandTime` the moment it did, the
//!      SAME failure mode total-T3 produced, for the SAME reason (a fixture in `tests/` this
//!      file's own criteria did not name as in-scope). Fixed the same way: extend the CLAIM below
//!      to assert `doc.expand_time`, not merely patch the fixture and move on.
//!   2. **`wat_doc::Category` has no arithmetic variant** — the closed set is
//!      `Transform | Reflection | ControlFlow | Binding | Clock | Arithmetic | Io | Probe | Combine`
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
/// `@Category Transform` is deliberately wrong-but-valid for an arithmetic example — see fact 2 in
/// the module header. The category is irrelevant to what this file measures.
fn doc_with(extra_tag_lines: &str) -> String {
    format!(
        "Add two i64s, yielding a declared fallback on overflow.\n\
         \n\
         @added         1.0.0\n\
         @Purity        Pure\n\
         @Determinism   Deterministic\n\
         @Total         Unreviewed\n\
         @ExpandTime    Unreviewed\n\
         @Category      Transform\n\
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

/// ★ THE CLAIM — every purity axis is read OFF THE DOC. Refutes the stale module header.
///
/// ⚠ FOUR axes now, not three. Arc 255 stone total-T3 minted `@Total` and made it REQUIRED, and
/// this probe's own baseline went RED on `MissingTotality` when it did — the file that exists to
/// assert "the axes are DECLARED" was itself not declaring the newest one. Extending the claim to
/// cover it, rather than only adding the directive to the fixture, is the difference between
/// fixing the probe and silencing it: a new axis that nothing here asserts is a new axis this
/// file's thesis has quietly stopped covering. Stone expand-T3 repeated the exact same lesson one
/// axis later — `@ExpandTime`'s own `MissingExpandTime` red — fixed the same way, below.
#[test]
fn axes_are_declared_not_derived() {
    let doc = wat_doc::parse(&doc_with("")).expect("baseline doc must parse");
    assert_eq!(doc.purity, Purity::Pure, "@Purity is parsed from the doc, not inferred");
    assert_eq!(
        doc.determinism,
        Determinism::Deterministic,
        "@Determinism is parsed from the doc, not inferred"
    );
    assert_eq!(
        doc.totality,
        wat_doc::Totality::Unreviewed,
        "@Total is parsed from the doc, not inferred"
    );
    assert_eq!(
        doc.expand_time,
        wat_doc::ExpandTime::Unreviewed,
        "@ExpandTime is parsed from the doc, not inferred"
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
