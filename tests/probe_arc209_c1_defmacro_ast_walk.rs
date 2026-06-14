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
//! Run: cargo test --release -p wat --test probe_arc209_c1_defmacro_ast_walk

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; PROOF 1 — a defmacro drives ast->children + drop + first on its Vector arg, returns a child.
;; PROGRAM-BODY path (top-level is a regular form, NOT a top-level quasiquote): the param `v`
;; is bound as a wat__WatAST node-value, so ast->children accepts it. `(:user::second-child
;; [10 20 30])` → children [10 20 30] → drop 1 → [20 30] → first → the `20` node → returned
;; directly (value_to_watast emits it) → the program sees literal 20.
(:wat::core::defmacro :user::second-child
  [v <- :wat::holon::HolonAST]
  -> :wat::holon::HolonAST
  (:wat::core::Option/expect -> :wat::WatAST
     (:wat::core::first (:wat::core::drop (:wat::core::ast->children v) 1))
     "second-child: need at least 2 children"))

(:wat::core::defn :user::probe-walk [] -> :wat::core::i64
  (:user::second-child [10 20 30]))

;; PROOF 2 — a defmacro rebuilds a Vector node via with-children, dropping the first element.
;; Program-body path again. `(:user::drop-first [10 20 30])` → with-children v (drop children 1)
;; → the `[20 30]` node → returned directly → a 2-element vector; length 2.
(:wat::core::defmacro :user::drop-first
  [v <- :wat::holon::HolonAST]
  -> :wat::holon::HolonAST
  (:wat::core::with-children v
     (:wat::core::drop (:wat::core::ast->children v) 1)))

(:wat::core::defn :user::probe-rebuild [] -> :wat::core::i64
  (:wat::core::Vector/length (:user::drop-first [10 20 30])))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn eval_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn defmacro_can_walk_arg_with_ast_children() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup: a defmacro using ast->children/drop/first must expand cleanly");
    let got = eval_i64(&world, "(:user::probe-walk)");
    assert!(
        matches!(got, Value::i64(20)),
        "expected 20: a defmacro must be able to ast->children its arg, drop a prefix, and \
         return a child node; got {got:?}"
    );
}

#[test]
fn defmacro_can_rebuild_node_with_children() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup: a defmacro using with-children must expand cleanly");
    let got = eval_i64(&world, "(:user::probe-rebuild)");
    assert!(
        matches!(got, Value::i64(2)),
        "expected 2: a defmacro must be able to with-children-rebuild a Vector node dropping a \
         prefix (the `s <- :State` triple drop); got {got:?}"
    );
}
