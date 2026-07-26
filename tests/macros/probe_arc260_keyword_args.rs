//! Arc 260 — disconfirming probe: wat has NO call-site keyword/named arguments (RED at HEAD).
//!
//! A user fn `sub [a b]` called with OUT-OF-ORDER keyword args `(:user::sub :b 3 :a 10)` should
//! reorder to positional `(sub 10 3)` → 7. At HEAD this is impossible — call args are positional
//! (`func.params.zip(args)`), so `:b 3 :a 10` reads as FOUR args to a 2-arg fn (arity/type error).
//! GREEN when option 1 (real keyword args) ships: the call-site reorders `:name val` → positional
//! by the callee's param names, before unification (check) and binding (eval).
//!
//! Wat source lives in the co-located fixture: probe_arc260_keyword_args.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc260_keyword_args -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
#[test]
#[ignore = "arc 260 RED — wat has no keyword args; call sites are positional. UN-IGNORE when \
            option 1 (call-site :name val reorder by param name) ships."]
fn user_fn_keyword_args_reorder_to_positional() {
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected 7: (:user::sub :b 3 :a 10) must reorder by param name to (sub 10 3) = 10-3; \
         got {got:?}"
    );
}
