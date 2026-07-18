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

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn ast_end_span_returns_position_past_close_paren() {
    // just-eval (rubric): `:user::end-col` lives in the co-located fixture.
    let got = call_beside(file!(), ":user::end-col")
        .unwrap_or_else(|e| panic!("end-col raised (ast-end-span undefined at HEAD): {e:?}"));
    let col = match got {
        Value::i64(n) => n,
        other => panic!("ast-end-span :col must be i64; got {other:?}"),
    };
    assert_eq!(
        col, 8,
        "ast-end-span of `(a b c)` must point one char past the `)` (col 8); got {col}"
    );
}
