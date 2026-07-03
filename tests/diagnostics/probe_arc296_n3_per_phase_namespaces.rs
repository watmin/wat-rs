//! Arc 296 N3 probe — each error FAMILY tags under its PHASE namespace, not the
//! uniform `#wat.kernel/`; shared value types (a nested `LoadFetchError`) STAY `wat.kernel`.
//!
//! Today every error serializes under `#wat.kernel/<Variant>`. N3 gives each top-level
//! family its phase namespace so a nested error chain reads its own phases:
//!   #wat.macro/… {:cause #wat.runtime/… {:… #wat.kernel/Location {…}}}
//! and `wat.kernel` stops being an error catch-all — it means "a shared value type".
//!
//! Behavioral RED (compiles at HEAD; names no new type — reads the written tag prefix):
//!   at HEAD every family writes `#wat.kernel/…` → the phase-ns assertions FAIL.
//!   after N3: CheckError→#wat.check, TypeError→#wat.type, RuntimeError→#wat.runtime,
//!   LoadError→#wat.load, while the embedded `LoadFetchError` stays `#wat.kernel/NotFound`.
//!
//! Committed `#[ignore]`'d (RED at HEAD); the N3 strike un-ignores it.

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind};
use wat::load::{LoadError, LoadErrorKind, LoadFetchError};
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat::types::error::{TypeError, TypeErrorKind};

fn make_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 1)
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn error_families_tag_under_their_phase_namespace() {
    // CheckError → #wat.check/
    let check = CheckError {
        span: make_span(),
        kind: CheckErrorKind::UnknownCallee { callee: ":user::do-thing".into() },
    };
    let s = wat_edn::write(&check.to_edn());
    assert_eq!(
        s,
        r#"#wat.check/UnknownCallee {:callee ":user::do-thing" :span {:file "test.wat" :line 1 :col 1}}"#,
        "CheckError must tag under #wat.check/"
    );

    // TypeError → #wat.type/
    let ty = TypeError {
        span: make_span(),
        kind: TypeErrorKind::DuplicateType { name: ":user::T".into() },
    };
    let s = wat_edn::write(&ty.to_edn());
    assert_eq!(
        s,
        r#"#wat.type/DuplicateType {:name ":user::T" :span {:file "test.wat" :line 1 :col 1}}"#,
        "TypeError must tag under #wat.type/"
    );

    // RuntimeError → #wat.runtime/
    let rt = RuntimeError {
        span: make_span(),
        kind: RuntimeErrorKind::UnboundSymbol("x".into()),
    };
    let s = wat_edn::write(&rt.to_edn());
    assert_eq!(
        s,
        r#"#wat.runtime/UnboundSymbol {:name "x" :span {:file "test.wat" :line 1 :col 1}}"#,
        "RuntimeError must tag under #wat.runtime/"
    );

    // LoadError → #wat.load/, AND the embedded LoadFetchError STAYS #wat.kernel/
    // (shared value types are cross-phase infra, not a phase error).
    let load = LoadError {
        span: make_span(),
        kind: LoadErrorKind::Fetch(LoadFetchError::NotFound("/no/such.wat".into())),
    };
    let s = wat_edn::write(&load.to_edn());
    // LoadError under #wat.load/; embedded LoadFetchError STAYS #wat.kernel/NotFound (shared infra).
    assert_eq!(
        s,
        r#"#wat.load/Fetch {:cause #wat.kernel/NotFound {:path "/no/such.wat"} :span {:file "test.wat" :line 1 :col 1}}"#,
        "LoadError must tag under #wat.load/; embedded LoadFetchError must stay #wat.kernel/"
    );
}
