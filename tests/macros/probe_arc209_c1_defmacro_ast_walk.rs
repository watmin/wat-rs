//! Arc 209 Stone C.1 — FOUNDATION probe: a DEFMACRO can drive the WatAST tooling on its arg.
//!
//! defservice's op-enum emission rests on one composition NOT proven anywhere on disk: a
//! `defmacro` receiving a Vector AST arg and using `ast->children` + `drop` + `with-children`
//! to rebuild it, then quasiquote-splicing the result. `fix.wat` proves those primitives — but
//! only in plain `defn`s; `cond` proves a defmacro walks args, but only with `first`/`rest`.
//! NO existing defmacro touches `ast->children`/`with-children`. This probe closes that gap so
//! the BRIEF cites a worked pattern, not an assumption (FM-2-bis).
//!
//! GREEN at HEAD is the GOAL here: this is a worked-reference probe (the composition the macro
//! engine ALREADY supports), not a disconfirming gate. If it were RED, `ast->children` in a
//! defmacro is a substrate gap — file that stone FIRST; do NOT brief C.1.
//!
//! Two proofs, the exact two moves defservice makes:
//!   1. `walk` — a defmacro reads its arg's children, drops a prefix, returns a child NODE.
//!      (= read the op-clause's arg-vec, index into it.)
//!   2. `rebuild` — a defmacro rebuilds a Vector node with a dropped prefix via `with-children`.
//!      (= drop the leading `s <- :State` triple → the variant's field vector.)
//!
//! Wat source lives in the co-located fixture: probe_arc209_c1_defmacro_ast_walk.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc209_c1_defmacro_ast_walk

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each proof is a zero-arg entry fn in the co-located fixture, driven via
// call_beside — no inline wat driver expression.

#[test]
fn defmacro_can_walk_arg_with_ast_children() {
    let got = call_beside(file!(), ":user::probe-walk")
        .expect("startup: a defmacro using ast->children/drop/first must expand cleanly");
    assert!(
        matches!(got, Value::i64(20)),
        "expected 20: a defmacro must be able to ast->children its arg, drop a prefix, and \
         return a child node; got {got:?}"
    );
}

#[test]
fn defmacro_can_rebuild_node_with_children() {
    let got = call_beside(file!(), ":user::probe-rebuild")
        .expect("startup: a defmacro using with-children must expand cleanly");
    assert!(
        matches!(got, Value::i64(2)),
        "expected 2: a defmacro must be able to with-children-rebuild a Vector node dropping a \
         prefix (the `s <- :State` triple drop); got {got:?}"
    );
}
