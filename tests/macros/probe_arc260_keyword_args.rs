//! Arc 260 — disconfirming probe: wat has NO call-site keyword/named arguments (RED at HEAD).
//!
//! A user fn `sub [a b]` called with OUT-OF-ORDER keyword args `(:user::sub :b 3 :a 10)` should
//! reorder to positional `(sub 10 3)` → 7. At HEAD this is impossible — call args are positional
//! (`func.params.zip(args)`), so `:b 3 :a 10` reads as FOUR args to a 2-arg fn (arity/type error).
//! GREEN when option 1 (real keyword args) ships: the call-site reorders `:name val` → positional
//! by the callee's param names, before unification (check) and binding (eval).
//!
//! PROBE FINDING (the load-bearing scope fact, grounded 2026-06-16):
//!   - USER fns retain param NAMES: `Function.params: Vec<String>` (eval, apply_function) and
//!     reachable at check via `sym.functions[path].params` — reorder is feasible on existing data.
//!   - INTRINSICS carry NO param names: a `TypeScheme` is `params: Vec<TypeExpr>` (types only,
//!     check.rs:79-81); `assertion-failed!`'s scheme (check.rs:14391) is `[string, Option<string>,
//!     Option<string>]` — nameless. The arc's TRIGGER (assertion-failed!) is therefore the HARDEST
//!     case: kwargs for intrinsics needs param-name infrastructure ADDED (a TypeScheme param_names
//!     field + populating the kernel-verb registrations + the intrinsic-call eval reorder).
//!   ⇒ Option 1 decomposes: user-fn kwargs first (names exist), then intrinsic kwargs (add names).
//!
//! Run: cargo test --release -p wat --test probe_arc260_keyword_args -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A user fn called with keyword args, deliberately OUT OF ORDER — only a real kwargs feature
// (reorder by param name) yields the right answer; positional reading gives the wrong one or errors.
const USER_FN_KWARGS: &str = r#"
(:wat::core::defn :user::sub [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::- a b))

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:user::sub :b 3 :a 10))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
#[ignore = "arc 260 RED — wat has no keyword args; call sites are positional. UN-IGNORE when \
            option 1 (call-site :name val reorder by param name) ships."]
fn user_fn_keyword_args_reorder_to_positional() {
    let world = startup_from_source(USER_FN_KWARGS, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once keyword args reorder by param name");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected 7: (:user::sub :b 3 :a 10) must reorder by param name to (sub 10 3) = 10-3; \
         got {got:?}"
    );
}
