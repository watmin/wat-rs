//! Arc 255 — FM-2-bis disconfirming probe: reflection PARITY between rust builtins
//! and user forms.
//!
//! THE ASK (builder): a reflection consumer must not tell a builtin from a user
//! form by the query path — `metadata-of` answers for BOTH, returning a uniform
//! map. Content is honest (a `:defined-in` tag declares rust vs wat), but the
//! mechanism is seamless.
//!
//! Today builtins are an opaque 454-arm dispatch `match` — registered nowhere,
//! reflected by nothing. And a bare user `defn` (no explicit metadata) returns
//! `None` from `metadata-of`. So NEITHER carries the guaranteed baseline.
//!
//! RED AT HEAD:
//!   - `(metadata-of :wat::core::i64::+)` → None (builtin not registered in sym).
//!   - `(metadata-of :my::f)` for a bare defn → None (no guaranteed baseline).
//! GREEN AFTER 255.1: both return `Some(baseline)` — the builtin registered into
//! `sym` as a `Native` Function entry; every registered form carrying the
//! auto-derived baseline (`:defined-in` + `:layer` + `:name` + `:arity`).
//!
//! Run un-ignored to confirm RED; sonnet un-ignores after 255.1 lands (and then
//! enriches these to assert the baseline KEYS, not just Some).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Freeze `src` (+ a nil main) and eval `(metadata-of <name_kw>)`; return whether
/// the result is `Some(_)` (i.e. the form carries reflectable metadata).
fn metadata_of_is_some(src: &str, name_kw: &str) -> bool {
    let full = format!("{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)", src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let call = format!("(:wat::runtime::metadata-of {})", name_kw);
    let ast = wat::parse_one_with_file(&call, "<probe>").expect("parse metadata-of call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("metadata-of eval").value_owned() {
        Value::Option(o) => o.is_some(),
        other => panic!("metadata-of must return Option; got {:?}", other),
    }
}

// RED at HEAD: a rust builtin is not registered → metadata-of returns None.
#[test]
fn metadata_of_answers_for_a_rust_builtin() {
    assert!(
        metadata_of_is_some("", ":wat::core::i64::+"),
        "metadata-of must answer (Some) for a rust builtin :wat::core::i64::+ — \
         seamless reflection parity with user forms. It returned None (builtins \
         are an opaque dispatch match, registered nowhere)."
    );
}

// RED at HEAD: a bare user defn has no guaranteed baseline → metadata-of None.
#[test]
fn user_form_carries_guaranteed_baseline() {
    let src = "(:wat::core::defn :my::f [x <- :wat::core::i64] -> :wat::core::i64 x)";
    assert!(
        metadata_of_is_some(src, ":my::f"),
        "metadata-of must answer (Some baseline) for a bare user defn — every \
         registered form carries the guaranteed baseline (:defined-in/:layer/\
         :name/:arity). It returned None."
    );
}
