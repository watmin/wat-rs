//! Arc 232 follow-on (the 6b-ii-β blocking dep) — generic-method TYPE-ARGUMENT APPLICATION.
//!
//! For `Locus/launch<S,R,St>` to mint its listener generically (`(listener' self :S :R)`), wat must:
//!   1. call a generic surface method with EXPLICIT type-args — `(:P/m<T1,T2> recv …)`;
//!   2. flow the method's type-params into the body as type-args to an intrinsic, so `:S`/`:R`
//!      resolve to the INSTANTIATED types, not the literal `Path(":S")`.
//! Generic FNS already do both (`foldl<T,Acc>`); generic METHODS (surface members) do not.
//!
//! GREEN as of Stone 6b-DEP (arc 272): `(:user::Mk/mk<wat::core::i64,wat::core::i64> …)` now strips
//! the `<…>` suffix to match the registered bare method name and binds the explicit type-args
//! `S=i64, R=i64` so the call checks under that substitution.
//! Full design: docs/arc/2026/06/272-…/DESIGN-STONE-6b-DEP-generic-method-type-application.md.

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn generic_method_called_with_explicit_type_args_mints_a_typed_bound() {
    let got = call_beside(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: (:user::Mk/mk<i64,i64> (thread)) resolved, the body's (listener' self :S :R) \
         instantiated S,R to i64,i64 and minted a Bound<i64,i64>; got {got:?}"
    );
}
