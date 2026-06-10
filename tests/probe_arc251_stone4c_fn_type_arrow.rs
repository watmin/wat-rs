//! FM 2-bis probe — arc 251 Stone 251.4c: `:->` function-type arrow `[i64 :-> i64]`.
//!
//! core.typed writes a function type as `[arg… :-> ret]`. wat today writes it as the
//! keyword `:wat::core::Fn(i64)->i64`. 251.4c adds the bracket form (a WatAST::Vector)
//! as a dual-read alias producing the same `TypeExpr::Fn`. `parse_type_node` gains a
//! Vector arm.
//!
//! HEAD-disconfirmation:
//! - C01: a `[wat.type/i64 :-> wat.type/i64]` fn-typed param ⇒ FAILS at HEAD
//!   (`parse_type_node` has no Vector arm — a `[…]` in a type slot is unparseable).
//!   Load-bearing: the param is passed to a sink fn typed with the KEYWORD fn-form,
//!   so the bracket must produce the SAME `TypeExpr::Fn` for unification.
//! - C02: the `:wat::core::Fn(...)->...` keyword spelling STILL checks (PRESERVATION;
//!   the keyword fn-type retires at 251.5).
//!
//! Post-251.4c: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc251_stone4c_fn_type_arrow`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn checks(decls: &str) -> Result<(), String> {
    let src = format!("{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// ─── C01: THE GAP — a `[… :-> …]` fn-type bracket type-checks ────────────────────

#[test]
fn contract_01_fn_type_bracket_checks() {
    // `[wat.type/i64 :-> wat.type/i64]` must produce the same Fn type as the keyword
    // spelling — proven by passing the bracket-typed param to a keyword-fn-typed sink.
    let r = checks(
        "(:wat::core::defn :user::sink \
           [h <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64 0)\n\
         (:wat::core::defn :user::pass [g <- [wat.type/i64 :-> wat.type/i64]] \
           -> :wat::core::i64 (:user::sink g))",
    );
    assert!(
        r.is_ok(),
        "[wat.type/i64 :-> wat.type/i64] must type-check as Fn(i64)->i64; got {r:?}"
    );
}

// ─── C02: PRESERVATION — the keyword Fn(...)->... spelling still checks ──────────

#[test]
fn contract_02_keyword_fn_type_still_checks() {
    assert!(
        checks(
            "(:wat::core::defn :user::id \
               [h <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64 0)"
        )
        .is_ok(),
        ":wat::core::Fn(...)->... keyword fn-type must keep type-checking"
    );
}
