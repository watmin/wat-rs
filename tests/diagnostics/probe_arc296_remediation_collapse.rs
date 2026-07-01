//! Arc 296 remediation collapse probe.
//!
//! Verifies that `collect_hints` / prose `:hint` is gone and that the single
//! `type_error_remedies` path surfaces structured `#wat.kernel/Remedy` values
//! for BOTH `TypeMismatch` and `ReturnTypeMismatch`.
//!
//! ## Contracts verified
//!
//! 1. A `TypeMismatch` on a retired callee (`:wat::core::vec`) emits `:remedies`
//!    containing `#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement …}`
//!    and NO `:hint` field.
//!
//! 2. A `TypeMismatch` whose `expected` contains `ProgramHandle<` and `got` contains
//!    `Thread<` (arc 114 shape mismatch) emits a `Remedy` whose `:note` carries the
//!    arc 114 migration guide — and NO `:hint` field.
//!
//! 3. A `ReturnTypeMismatch` on a retired callee emits `:remedies` with the retirement
//!    entry and NO `:hint` field.

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat_edn::OwnedValue;

fn make_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 1)
}

// ─── Shared assertion helpers ─────────────────────────────────────────────────

fn assert_no_hint(s: &str) {
    assert!(
        !s.contains(":hint"),
        "`:hint` field must be absent after arc 296 remediation collapse; got: {}",
        s
    );
}

fn assert_remedies_vector(edn: &OwnedValue) -> &[OwnedValue] {
    if let OwnedValue::Tagged(_, body) = edn {
        if let OwnedValue::Map(fields) = body.as_ref() {
            let remedies_field = fields.iter().find(|(k, _)| {
                matches!(k, OwnedValue::Keyword(kw) if kw.name() == "remedies")
            });
            let (_, remedies_val) = remedies_field
                .expect("`:remedies` field must be present in TypeMismatch EDN");
            if let OwnedValue::Vector(items) = remedies_val {
                return items.as_slice();
            }
            panic!(":remedies must be a Vector; got: {:?}", remedies_val);
        }
    }
    panic!("expected Tagged EDN with Map body; got: {:?}", edn);
}

// ─── Probe 1 — TypeMismatch on retired callee: :remedies + no :hint ──────────

#[test]
fn probe_1_type_mismatch_retired_callee_emits_remedies_not_hint() {
    let err = CheckError {
        span: make_span(),
        kind: CheckErrorKind::TypeMismatch {
            callee: ":wat::core::vec".into(),
            param: "x".into(),
            expected: ":wat::core::Vector<:wat::core::i64>".into(),
            got: ":wat::core::String".into(),
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);
    eprintln!("=== probe_1 TypeMismatch(vec): {}", s);

    // No prose :hint field.
    assert_no_hint(&s);

    // :remedies must be present.
    assert!(s.contains(":remedies"), ":remedies must be present; got: {}", s);

    // At least one remedy.
    let items = assert_remedies_vector(&edn);
    assert!(
        !items.is_empty(),
        ":remedies must be non-empty for retired callee :wat::core::vec; got: {}",
        s
    );

    // First remedy must be the retirement entry: form = :wat::core::Vector.
    let first_str = wat_edn::write(&items[0]);
    eprintln!("=== probe_1 first remedy: {}", first_str);
    assert!(
        first_str.contains("#wat.kernel/Remedy"),
        "first remedy must be #wat.kernel/Remedy; got: {}",
        first_str
    );
    assert!(
        first_str.contains(":wat::core::Vector"),
        "first remedy :form must be :wat::core::Vector; got: {}",
        first_str
    );
    assert!(
        first_str.contains(":kind :retirement"),
        "first remedy :kind must be :retirement; got: {}",
        first_str
    );

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 2 — TypeMismatch with ProgramHandle↔Thread shape mismatch ─────────

#[test]
fn probe_2_type_mismatch_arc114_shape_emits_spawn_thread_remedy_not_hint() {
    let err = CheckError {
        span: make_span(),
        kind: CheckErrorKind::TypeMismatch {
            callee: ":some::fn".into(),
            param: "handle".into(),
            // expected contains ProgramHandle<, got contains Thread< — arc 114 shape trigger
            expected: ":wat::kernel::ProgramHandle<:wat::core::String>".into(),
            got: ":wat::kernel::Thread<:wat::core::nil,:wat::core::String>".into(),
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);
    eprintln!("=== probe_2 TypeMismatch(arc114 shape): {}", s);

    // No prose :hint field.
    assert_no_hint(&s);

    // :remedies must be present.
    assert!(s.contains(":remedies"), ":remedies must be present; got: {}", s);

    // At least one remedy.
    let items = assert_remedies_vector(&edn);
    assert!(
        !items.is_empty(),
        ":remedies must be non-empty for arc 114 shape mismatch; got: {}",
        s
    );

    // The remedy must reference :wat::kernel::spawn-thread.
    let first_str = wat_edn::write(&items[0]);
    eprintln!("=== probe_2 first remedy: {}", first_str);
    assert!(
        first_str.contains(":wat::kernel::spawn-thread"),
        "arc 114 shape remedy must reference :wat::kernel::spawn-thread; got: {}",
        first_str
    );
    assert!(
        first_str.contains(":kind :retirement"),
        "arc 114 shape remedy must have :kind :retirement; got: {}",
        first_str
    );
    // The :note must carry the arc 114 migration guidance.
    assert!(
        first_str.contains("arc 114"),
        "arc 114 remedy :note must carry the arc 114 guide; got: {}",
        first_str
    );

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 3 — ReturnTypeMismatch on retired callee: :remedies + no :hint ────

#[test]
fn probe_3_return_type_mismatch_retired_callee_emits_remedies_not_hint() {
    // ReturnTypeMismatch with no stored remedies; the retirement lookup fires
    // on the function name ":wat::core::vec" (retired).
    let err = CheckError {
        span: make_span(),
        kind: CheckErrorKind::ReturnTypeMismatch {
            function: ":wat::core::list".into(),
            expected: ":wat::core::Vector<:wat::core::i64>".into(),
            got: ":wat::core::nil".into(),
            remedies: vec![],
        },
    };

    let edn = err.to_edn();
    let s = wat_edn::write(&edn);
    eprintln!("=== probe_3 ReturnTypeMismatch(list): {}", s);

    // No prose :hint field.
    assert_no_hint(&s);

    // :remedies must be present.
    assert!(s.contains(":remedies"), ":remedies must be present; got: {}", s);

    // At least one remedy — the retirement table entry for :wat::core::list.
    let items = assert_remedies_vector(&edn);
    assert!(
        !items.is_empty(),
        ":remedies must be non-empty for retired function :wat::core::list; got: {}",
        s
    );

    let first_str = wat_edn::write(&items[0]);
    eprintln!("=== probe_3 first remedy: {}", first_str);
    assert!(
        first_str.contains(":wat::core::Vector"),
        "retirement remedy :form must be :wat::core::Vector; got: {}",
        first_str
    );
    assert!(
        first_str.contains(":kind :retirement"),
        "retirement remedy :kind must be :retirement; got: {}",
        first_str
    );

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}
