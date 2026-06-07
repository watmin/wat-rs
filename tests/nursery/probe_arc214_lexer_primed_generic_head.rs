//! Arc 214 Stone 4.6a-i prereq — primed type heads with multi-param generics
//! must lex (FM-2-bis disconfirming probe).
//!
//! The 4.5 peer types are PRIMED (`:wat::kernel::Thread'` / `Process'`) and
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
//! Run: `cargo test --release --test nursery probe_arc214_lexer_primed_generic_head`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn startup_with_param_type(ty: &str) -> Result<(), String> {
    let src = format!(
        "(:wat::core::defn :user::take [m <- {}] -> :wat::core::nil nil)\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        ty
    );
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{}", e))
}

/// Control: unprimed two-param generic lexes today (the live test.wat shape).
#[test]
fn control_unprimed_two_param_lexes() {
    // NOTE: uses the EXISTING :wat::kernel::Thread type (live in wat/test.wat).
    startup_with_param_type(":wat::kernel::Thread<wat::core::nil,wat::core::nil>")
        .expect("unprimed Thread<nil,nil> must lex + check (live shape, test.wat:707)");
}

/// LOAD-BEARING: a PRIMED head with two params must LEX. The check may still
/// reject the unregistered type — the assertion is only that the failure is
/// NOT the lexer's CommaInKeywordBody.
#[test]
fn primed_two_param_must_lex() {
    match startup_with_param_type(":wat::kernel::Thread'<wat::core::i64,wat::core::i64>") {
        Ok(()) => {} // lexed and checked — fine
        Err(e) => {
            assert!(
                !e.contains("comma inside keyword body"),
                "primed generic head must pass the LEXER; got CommaInKeywordBody:\n{}",
                e
            );
        }
    }
}

/// LOAD-BEARING twin: whitespace inside a primed generic must also lex
/// (angle_depth tracking, not just comma tolerance).
#[test]
fn primed_two_param_with_space_must_lex() {
    match startup_with_param_type(":wat::kernel::Thread'<wat::core::i64, wat::core::i64>") {
        Ok(()) => {}
        Err(e) => {
            assert!(
                !e.contains("comma inside keyword body") && !e.contains("Unclosed"),
                "primed generic head with space must pass the LEXER; got:\n{}",
                e
            );
        }
    }
}
