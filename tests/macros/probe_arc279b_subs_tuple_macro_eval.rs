//! Arc 279.1 — FOUNDATION probe (the worked reference the format rewrite copies).
//!
//! Proves the macro-eval engine can run the exact mechanics the `{{`/`}}` tokenizer needs:
//!   1. `string::subs` at EXPAND TIME (newly added to is_pure_total) — char-walk via
//!      `(map (fn [i] (subs s i (+ i 1))) (range 0 (length s)))`.
//!   2. A `Tuple` heterogeneous ACCUMULATOR threaded through `foldl`, accessed via first/second.
//!   3. Per-char `=` compares + `if` driving state, at expand time.
//!
//! Wat source lives in the co-located fixture: probe_arc279b_subs_tuple_macro_eval.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc279b_subs_tuple_macro_eval

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn subs_tuple_char_walk_runs_at_macro_eval() {
    let world = startup_beside(file!())
        .expect("strip-braces macro (subs + Tuple foldl) must expand cleanly at compile time");
    let ast = wat::parse_one!("(:user::probe)").expect("parse the defn call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("probe must eval")
        .value_owned();
    let s = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected String; got {other:?}"),
    };
    // "a{b{c" → kept "abc", dropped 2 braces → "abc|2".
    assert_eq!(s, "abc|2", "subs char-walk + Tuple foldl at expand time; got {s:?}");
}
