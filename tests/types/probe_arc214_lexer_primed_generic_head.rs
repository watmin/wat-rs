//! Arc 214 Stone 4.6a-i prereq — primed type heads with multi-param generics
//! must lex (FM-2-bis disconfirming probe).
//!
//! The 4.5 peer types are PRIMED (`:wat::kernel::Thread` / `Process'`) and
//! parametric (`<I,O>`). The lexer's `<`-as-type-head detection
//! (src/lexer.rs `lex_keyword`) increments `angle_depth` only when the char
//! before `<` is alphanumeric/`_` — an apostrophe-suffixed head (`Thread'<`)
//! is missed, so the comma between the params hits `CommaInKeywordBody`.
//!
//! Disambiguation safety: operator `<` in a keyword path always follows `::`
//! (`:wat::core::<`), and arc-171 discriminator apostrophes come AFTER an op
//! name (`<'2`) — so `'` immediately before `<` can only be a primed type
//! head. `parse_type_expr` already accepts the primed parametric form; only
//! the source lexer lags.
//!
//! Controls prove the isolation: unprimed multi-param and primed single-param
//! both lex today; ONLY primed + comma fails.
//!
//! Wat fixtures: tests/types/probe_arc214_lexer_primed_generic_head_{control,primed}.wat (positive),
//!   tests/types/probe_arc214_lexer_primed_generic_head_{primed_space,unprimed_space}.wat.bad (negative).
//!
//! Run: `cargo nextest run --release -E 'binary(types)' -F probe_arc214_lexer_primed_generic_head`

use wat::freeze::startup_from_file;

/// Control: unprimed two-param generic lexes today (the live test.wat shape).
#[test]
fn control_unprimed_two_param_lexes() {
    // The name is neutral by design — the subject is the LEXER, which never consults
    // the type registry. Borrowing a live (or dead) kernel type here proves nothing.
    startup_from_file(
        "tests/types/probe_arc214_lexer_primed_generic_head_control.wat",
    )
    .expect("unprimed two-param generic head must lex + check");
}

/// LOAD-BEARING: a PRIMED head with two params must LEX. The check may still
/// reject the unregistered type — the assertion is only that the failure is
/// NOT the lexer's CommaInKeywordBody.
#[test]
fn primed_two_param_must_lex() {
    match startup_from_file("tests/types/probe_arc214_lexer_primed_generic_head_primed.wat")
        .map(|_| ())
        .map_err(|e| format!("{}", e))
    {
        Ok(()) => {} // lexed and checked — fine
        Err(e) => {
            // rune:lint(loose-assert) — targeted absence in Err arm; at HEAD startup returns Ok (primed head lexes and checks), so this arm is unreachable; the assert guards against a lexer regression where CommaInKeywordBody would fire; the specific error message varies by which check phase rejects
            assert!(
                !e.contains("comma inside keyword body"),
                "primed generic head must pass the LEXER; got CommaInKeywordBody:\n{}",
                e
            );
        }
    }
}

/// PARITY twin — **RE-POINTED, arc 109 "annihilate the angle bracket".**
///
/// This test's original subject (whitespace inside an OPEN `<...>` type head
/// producing `UnclosedBracketInKeyword`, never the apostrophe-specific
/// `CommaInKeywordBody`) no longer exists: `<` opening a type head at all is
/// now refused at the FIRST `<` — for `HashMap'<wat::core::i64 >` and
/// `HashMap<wat::core::nil >` alike, the wall fires before the lexer ever
/// reaches the interior space, primed or not. The permission these fixtures
/// exercised is gone, so the honest re-point is the wall itself: both the
/// primed and unprimed heads are refused, by the SAME mechanism
/// (`AngleTypeHeadInName`), which is the parity claim this test still makes —
/// only the mechanism moved. Assert the MECHANISM (not the whole diagnostic,
/// whose byte offset is fixture-fragile) per the arc's own precedent.
#[test]
fn primed_two_param_with_space_fails_same_as_unprimed() {
    let primed = startup_from_file(
        "tests/types/probe_arc214_lexer_primed_generic_head_primed_space.wat.bad",
    )
    .map(|_| ())
    .expect_err("a primed angle type head must be refused — arc 109");
    let primed = format!("{}", primed);
    let unprimed = startup_from_file(
        "tests/types/probe_arc214_lexer_primed_generic_head_unprimed_space.wat.bad",
    )
    .map(|_| ())
    .expect_err("an unprimed angle type head must be refused — arc 109 (unprimed control)");
    let unprimed = format!("{}", unprimed);
    // rune:lint(loose-assert) — targeted PRESENCE over a large structured diagnostic; the
    // assertion names the MECHANISM (the arc 109 wall), which is the parity claim this test
    // makes. An exact-match golden would pin the lex error's byte offset and re-break on every
    // unrelated edit to the fixture — same reasoning as probe_arc232_generic_method_type_
    // application.rs's precedent.
    assert!(
        primed.contains("annihilate the angle bracket"),
        "primed head must be refused by the arc 109 angle wall; got: {primed}"
    );
    // rune:lint(loose-assert) — same reasoning as the primed assertion above: targeted
    // PRESENCE of the mechanism, not the whole diagnostic.
    assert!(
        unprimed.contains("annihilate the angle bracket"),
        "unprimed head must be refused by the SAME wall (parity); got: {unprimed}"
    );
}
