//! Arc 232 follow-on (the 6b-ii-β blocking dep) — generic-method TYPE-ARGUMENT APPLICATION.
//!
//! For `Host/launch<S,R,St>` to mint its listener generically (`(listener' self :S :R)`), wat must:
//!   1. call a generic protocol method with EXPLICIT type-args — `(:P/m<T1,T2> recv …)`;
//!   2. flow the method's type-params into the body as type-args to an intrinsic, so `:S`/`:R`
//!      resolve to the INSTANTIATED types, not the literal `Path(":S")`.
//! Generic FNS already do both (`foldl<T,Acc>`); generic METHODS do not.
//!
//! RED at HEAD `82b21ce8`: `(:user::Mk/mk<wat::core::i64,wat::core::i64> …)` → check error
//! `unknown callee: :user::Mk/mk<wat::core::i64,wat::core::i64>` (the call-head resolver doesn't strip
//! the `<…>` suffix to match the method name). `#[ignore]` until the dep lands; UN-IGNORE then.
//! Full design: docs/arc/2026/06/272-…/DESIGN-STONE-6b-DEP-generic-method-type-application.md.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defprotocol :user::Mk
  (mk<S,R> [self <- :user::Mk] -> :wat::spawn::Bound<S,R>))

;; the impl body instantiates the method's type-params S,R as type-args to listener'.
(:wat::core::extend-type :wat::spawn::ThreadOpts :user::Mk
  (mk [self] (:wat::kernel::listener' self :S :R)))

;; calling with explicit type-args <i64,i64>; if the Bound minted, return 42.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [b (:user::Mk/mk<wat::core::i64,wat::core::i64> (:wat::spawn::thread))]
    (:wat::core::let [_ (:wat::spawn::Bound/listener b)] 42)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
#[ignore = "arc-232 follow-on RED: generic method called with explicit <T,T> type-args resolves as \
            'unknown callee'; the type-param→listener'-type-arg flow is unbuilt. Blocks 6b-ii-β. \
            UN-IGNORE when the dep lands."]
fn generic_method_called_with_explicit_type_args_mints_a_typed_bound() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once generic-method type-application is built");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: (:user::Mk/mk<i64,i64> (thread)) resolved, the body's (listener' self :S :R) \
         instantiated S,R to i64,i64 and minted a Bound<i64,i64>; got {got:?}"
    );
}
