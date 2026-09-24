//! Arc 255 Stone 255.24 (C-b2) — **defservice declares the type parameters it emits.**
//!
//! Every generic `defn` defservice emits names the service's own parameters (`K`, `V`) and/or the
//! transport letter (`T`, or `Xt` when the service binds `T` itself). Before this stone none of
//! them declared those letters: they were type parameters by SPELLING (`is_type_param_letter`).
//! Now each generated `defn` carries `:- [..]` naming exactly the letters its signature uses.
//!
//! Rows:
//! - the emitted-form census (`:probe::emitted-defn-binders`, a form-level expansion of one
//!   monomorphic and one `:- [K V]` service). On the pre-stone binary (`56b57f09e`) every row
//!   below whose binder is not `-` read `-`.
//! - `start$impl` of a `:- [K V]` service, called with a thread locus, is
//!   `(Handle :- [String i64 Shared])`; a process locus gives `… Wire`; a claim of the other
//!   transport is refused. (These already held on the pre-stone binary — the undeclared letter was
//!   generalized by spelling; they pin that the DECLARED binder keeps them.)
//!
//! Run: cargo test --release -p wat --test services -- arc255_24

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{call_beside_value, startup_beside, startup_from_file, StartupError};
use wat::runtime::Value;

const REFUSED: &str = "tests/services/probe_arc255_24_start_impl_claims_the_other_transport.wat.bad";

/// The monomorphic service: only the transport letter `T` can be free.
const MONO: &[&str] = &[
    ":probe::mono-svc::serve [T]",
    ":probe::mono-svc::init -",
    ":probe::mono-svc::stop-project -",
    ":probe::mono-svc::hibernate-project -",
    ":probe::mono-svc::dispatch-admin -",
    ":probe::mono-svc::extract-addr [T]",
    ":probe::mono-svc/get -",
    ":probe::mono-svc/stop [T]",
    ":probe::mono-svc/hibernate [T]",
    ":probe::mono-svc/grant [T]",
    ":probe::mono-svc/revoke [T]",
    ":probe::mono-svc::service-forms -",
    ":probe::mono-svc/start$impl [T]",
    ":probe::mono-svc/start$impl-thread -",
    ":probe::mono-svc/start$impl-process -",
    ":probe::mono-svc/resume$impl [T]",
    ":probe::mono-svc/resume$impl-thread -",
    ":probe::mono-svc/resume$impl-process -",
];

/// The `:- [K V]` service: `K`/`V` everywhere its types reach, plus `T` where the transport does.
const PARA: &[&str] = &[
    ":probe::pair-svc::serve [K V T]",
    ":probe::pair-svc::init [K V]",
    ":probe::pair-svc::stop-project [K V]",
    ":probe::pair-svc::hibernate-project [K V]",
    ":probe::pair-svc::dispatch-admin [K V]",
    ":probe::pair-svc::extract-addr [K V T]",
    ":probe::pair-svc/put [K V]",
    ":probe::pair-svc/stop [K V T]",
    ":probe::pair-svc/hibernate [K V T]",
    ":probe::pair-svc/grant [K V T]",
    ":probe::pair-svc/revoke [K V T]",
    ":probe::pair-svc::service-forms -",
    ":probe::pair-svc/start$impl [K V T]",
    ":probe::pair-svc/start$impl-thread [K V]",
    ":probe::pair-svc/start$impl-process [K V]",
    ":probe::pair-svc/resume$impl [K V T]",
    ":probe::pair-svc/resume$impl-thread [K V]",
    ":probe::pair-svc/resume$impl-process [K V]",
];

fn strings(v: &Value) -> Vec<String> {
    match v {
        Value::Vec(items) => items
            .iter()
            .map(|i| match i {
                Value::String(s) => s.to_string(),
                other => panic!("expected a String row; got {other:?}"),
            })
            .collect(),
        other => panic!("expected a Vector of rows; got {other:?}"),
    }
}

#[test]
fn arc255_24_every_emitted_defn_declares_its_letters() {
    let got = call_beside_value(file!(), ":probe::emitted-defn-binders")
        .unwrap_or_else(|e| panic!("emitted-defn-binders raised: {e:?}"));
    let Value::Tuple(pair) = got else { panic!("expected a (mono, para) Tuple; got {got:?}") };
    assert_eq!(strings(&pair[0]), MONO, "monomorphic service: emitted defn binders");
    assert_eq!(strings(&pair[1]), PARA, "`:- [K V]` service: emitted defn binders");
}

#[test]
fn arc255_24_start_impl_names_the_locus_transport() {
    startup_beside(file!()).expect(
        "start$impl of a `:- [K V]` service: thread → (Handle :- [String i64 Shared]), process → \
         … Wire, and an abstract (Locus :- [T]) passes T through",
    );
}

#[test]
fn arc255_24_start_impl_claim_of_the_other_transport_is_refused() {
    let err = startup_from_file(REFUSED).expect_err("a claim of the other transport must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // rune:lint(no-inlined-wat) — golden COMPARISON text for a ReturnTypeMismatch's rendered
    // `expected`/`got` fields, never a wat world/driver; nothing builds or runs a program from it.
    let handle = |t: &str| {
        format!("(:probe::pair-svc::Handle :- [:wat::core::String :wat::core::i64 {t}])")
    };
    for (func, claimed, actual) in [
        (":probe::thread-claimed-wire", ":wat::kernel::Wire", ":wat::kernel::Shared"),
        (":probe::process-claimed-shared", ":wat::kernel::Shared", ":wat::kernel::Wire"),
    ] {
        let (want_expected, want_got) = (handle(claimed), handle(actual));
        wat::assert_check_error_present!(errs,
            CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
                if function == func && *expected == want_expected && *got == want_got);
    }
}
