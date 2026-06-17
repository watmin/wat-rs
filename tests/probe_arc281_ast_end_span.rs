//! Arc 281 — disconfirming probe: there is no `ast-end-span` (RED at HEAD).
//!
//! `ast-span` returns a node's START `{:line, :col}` (src START location). For a STRUCTURAL node (a
//! whole `(...)` form) an auto-fix needs the END too — to compute `old-len = end-offset - start-offset`
//! and replace the entire form. `:wat::core::ast-end-span` returns the END `{:line, :col}` — the
//! position one char PAST the node's last char (for `(a b c)`, the col just after `)`).
//!
//! This is the KEYSTONE: it unblocks every structural auto-fix (277.1b ladder fix, the concat→format
//! fix, the sweep). At HEAD `ast-end-span` is undefined → startup/eval errors → RED. GREEN when arc 281
//! ships end-position tracking (lexer → SpannedToken → Span → parser) + the intrinsic.
//!
//! Run: cargo test --release -p wat --test probe_arc281_ast_end_span -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// read-string "(a b c)" → the parse tree; first top-level form is the (a b c) list.
// ast-span of that list = {:line 1 :col 1} (the `(`); ast-end-span must = {:line 1 :col 8}
// (one past the `)` at col 7). The probe returns the end :col; RED until the intrinsic exists.
const PROGRAM: &str = r#"
(:wat::core::defn :user::end-col [] -> :wat::core::i64
  (:wat::core::let
    [tree (:wat::core::read-string "(a b c)")
     form (:wat::core::Option/expect -> :wat::WatAST
            (:wat::core::first (:wat::core::ast->children tree))
            "first form")
     espan (:wat::core::ast-end-span form)]
    (:wat::core::Option/expect -> :wat::core::i64
      (:wat::core::HashMap/get espan :col)
      "end :col")))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
#[ignore = "arc 281 — RED until ast-end-span ships; un-ignore on green"]
fn ast_end_span_returns_position_past_close_paren() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup: ast-end-span must be defined once arc 281 ships");
    let ast = wat::parse_one!("(:user::end-col)").expect("parse the defn call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("end-col raised (ast-end-span undefined at HEAD): {e:?}"))
        .value_owned();
    let col = match got {
        Value::i64(n) => n,
        other => panic!("ast-end-span :col must be i64; got {other:?}"),
    };
    assert_eq!(
        col, 8,
        "ast-end-span of `(a b c)` must point one char past the `)` (col 8); got {col}"
    );
}
