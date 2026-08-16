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

/// PARITY twin: whitespace inside `<...>` is a lex error BY DESIGN (keywords
/// cannot contain whitespace — same rule for unprimed heads). The honest
/// assertion is that a primed head fails the SAME way (unclosed-bracket),
/// never the apostrophe-specific `CommaInKeywordBody` — i.e. angle_depth is
/// tracked for the primed head, so the comma is protected and the whitespace
/// rule is what fires. (The earlier form of this test asserted "must lex,"
/// which mistook the by-design whitespace rule for the apostrophe bug and
/// passed only via a case-sensitivity accident; corrected 2026-06-07.)
#[test]
fn primed_two_param_with_space_fails_same_as_unprimed() {
    let primed = startup_from_file(
        "tests/types/probe_arc214_lexer_primed_generic_head_primed_space.wat.bad",
    )
    .expect_err("whitespace inside <...> is a lex error by design");
    let primed = format!("{}", primed);
    let unprimed = startup_from_file(
        "tests/types/probe_arc214_lexer_primed_generic_head_unprimed_space.wat.bad",
    )
    .expect_err("whitespace inside <...> is a lex error by design (unprimed control)");
    let unprimed = format!("{}", unprimed);
    wat::assert_edn_matches_file!(primed, "probe_arc214_lexer_primed_generic_head__primed_two_param_with_space_fails_same_as_unprimed__primed.edn", "primed head, whitespace-in-keyword lex error");
    wat::assert_edn_matches_file!(unprimed, "probe_arc214_lexer_primed_generic_head__primed_two_param_with_space_fails_same_as_unprimed__unprimed.edn", "unprimed control, whitespace-in-keyword lex error");
}
