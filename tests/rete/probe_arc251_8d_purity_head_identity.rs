//! Arc 251 stone 251.8d-ii (FIFTH draw) — **the purity head reads one identity, not one spelling.**
//!
//! `src/rete/purity.rs`'s `classify_expr` read a form head as either payload, RAW:
//! `Keyword(k) => k.as_str()` / `Symbol(id) => id.as_str()`. Everything downstream is keyed on
//! the INTERNAL identity — `intrinsic_meta`'s table, `effectful_by_prefix`'s prefixes,
//! `rete_op_for`'s rows — so the faithful-Clojure spelling of a *proven* verb was mis-keyed and
//! the DEFAULT-DENY fired on a verb that is in the table. 255.13's ledger calls that shape **B**,
//! *dual-raw*, and names it strictly worse than a keyword-only read: the walk sees the symbol and
//! then mis-keys it.
//!
//! ⛔⛔ **THE CURE RUNS IN THE PERMISSIVE DIRECTION.** A purity gate is default-deny, so teaching
//! it a spelling makes it accept MORE — the red → false-green direction. These rows are the stone.
//! Measured on two binaries built from one tree (`git stash push -- src/rete/purity.rs`):
//!
//! ```text
//!                         PRE-CURE   POST-CURE
//!   lt-kw-pure?             true       true      control — the keyword side may not move
//!   lt-sym-pure?           FALSE       true      ⭐ THE CURE
//!   println-kw-pure?        false      false     adversarial: genuinely impure, keyword
//!   println-sym-pure?       false      false     ⭐ adversarial: genuinely impure, SYMBOL
//!   io-kw/io-sym-pure?      false      false     a second impure prefix
//!   unknown-kw/sym-pure?    false      false     ⭐ unknown stays unknown
//!   bare-sym-pure?          false      false     a namespace-less symbol mints no identity
//!   uuid-kw-pure?           true       true
//!   uuid-sym-pure?         FALSE       true      ⭐ THE CURE, a second verb
//!   uuid-kw/sym-det?        false      false     ⭐ THE AXES DO NOT COLLAPSE
//!   lt-kw/sym-primitive?    false      false     ⭐ LAW A IS NOT LOOSENED
//!   rete-kw-primitive?      true       true
//!   rete-sym-primitive?    FALSE       true      ⭐ THE CURE, the rete vocabulary
//! ```
//!
//! **Exactly three rows flip and all three flip toward `true`.** Every refusal row is green on
//! BOTH binaries, which is what makes them a test of the WALL rather than a test of the cure.
//!
//! Run: `cargo nextest run --release -E 'binary(rete) and test(purity_head_identity)'`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn ask(entry: &str) -> bool {
    match call_beside_value(file!(), entry).expect("the fixture must evaluate") {
        Value::bool(b) => b,
        other => panic!("{entry} answered {other:?}, not a bool"),
    }
}

/// ⭐ THE THREE NON-VACUITY FAMILIES, IN ONE TEST — because the claim is about the JOINT
/// behaviour of the door, and a per-row test would let a cure that admits everything pass three
/// of them. The table is asserted whole so a failure prints every row, not the first one.
#[test]
fn the_purity_head_reads_one_identity_and_still_refuses_what_it_should() {
    let rows: &[(&str, bool, &str)] = &[
        // (a) THE CURE — a proven-pure verb, both spellings, same answer.
        (":user::lt-kw-pure?", true, "control: the keyword spelling may not move"),
        (":user::lt-sym-pure?", true, "THE CURE: wat.core/< is the same verb as :wat::core::<"),
        (":user::uuid-kw-pure?", true, "control"),
        (":user::uuid-sym-pure?", true, "THE CURE, a second verb"),
        // (b) ADVERSARIAL — a genuinely IMPURE verb is still refused in BOTH spellings.
        (":user::println-kw-pure?", false, "effectful_by_prefix :wat::kernel::"),
        (":user::println-sym-pure?", false, "ADVERSARIAL: wat.kernel/println must stay refused"),
        (":user::io-kw-pure?", false, "effectful_by_prefix :wat::io::"),
        (":user::io-sym-pure?", false, "ADVERSARIAL: wat.io/read-file must stay refused"),
        // (c) An UNKNOWN head is still unknown — the cure is not "anything I can't parse is fine".
        (":user::unknown-kw-pure?", false, "no table row"),
        (":user::unknown-sym-pure?", false, "ADVERSARIAL: re-spelling does not mint membership"),
        (":user::bare-sym-pure?", false, "a namespace-less symbol names nothing"),
        // (d) THE AXES DO NOT COLLAPSE — Uuid/v4 is pure AND non-deterministic, both spellings.
        (":user::uuid-kw-det?", false, "v4 is random"),
        (":user::uuid-sym-det?", false, "ADVERSARIAL: the door re-spells a name, it does not say yes"),
        // (e) LAW A IS NOT LOOSENED — a core-spelled op is not a rete primitive, either spelling.
        (":user::lt-kw-primitive?", false, "law A refuses :wat::core::< in a where"),
        (":user::lt-sym-primitive?", false, "ADVERSARIAL: and it refuses wat.core/< too"),
        // …and the positive control that keeps (e) from passing on a gate that refuses everything.
        (":user::rete-kw-primitive?", true, "a RETE_OPS row is admitted"),
        (":user::rete-sym-primitive?", true, "THE CURE, through the rete vocabulary"),
    ];
    let wrong: Vec<String> = rows
        .iter()
        .filter(|(entry, want, _)| ask(entry) != *want)
        .map(|(entry, want, why)| format!("  {entry} → {} (want {want}) — {why}", !want))
        .collect();
    assert!(
        wrong.is_empty(),
        "the purity head's identity door answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n"),
    );
}
