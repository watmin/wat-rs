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

    // Must be exact EDN: :remedies Vector with retirement entry; no :hint.
    assert_eq!(
        s,
        r#"#wat.check/TypeMismatch {:callee ":wat::core::vec" :param "x" :expected ":wat::core::Vector<:wat::core::i64>" :got ":wat::core::String" :remedies [#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same Vec<T> value"}] :span {:file "test.wat" :line 1 :col 1}}"#,
        "TypeMismatch on retired callee must emit structured :remedies Vector (NO :hint)"
    );

    let items = assert_remedies_vector(&edn);
    assert!(!items.is_empty(), ":remedies must be non-empty for retired callee :wat::core::vec");

    let first_str = wat_edn::write(&items[0]);
    assert_eq!(
        first_str,
        r#"#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "rename `:wat::core::vec` → `:wat::core::Vector` (verb-equals-type, arc 109 slice 1f); substrate produces the same Vec<T> value"}"#,
        "first remedy must be exact #wat.kernel/Remedy with :kind :retirement"
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

    // Must be exact EDN: :remedies with arc 114 spawn-thread retirement remedy; no :hint.
    assert_eq!(
        s,
        r#"#wat.check/TypeMismatch {:callee ":some::fn" :param "handle" :expected ":wat::kernel::ProgramHandle<:wat::core::String>" :got ":wat::kernel::Thread<:wat::core::nil,:wat::core::String>" :remedies [#wat.kernel/Remedy {:form ":wat::kernel::spawn-thread" :kind :retirement :score 0 :note "arc 114 — :wat::kernel::spawn / :wat::kernel::join / :wat::kernel::join-result retire. Programs deliver values only via their output channel; R-via-join is gone. Migrate: (:wat::kernel::spawn :worker args...) → (:wat::kernel::spawn-thread (:wat::core::fn ((_in :rust::crossbeam_channel::Receiver<()>) (_out :rust::crossbeam_channel::Sender<()>)) (:worker args...))) returning :wat::kernel::Thread<(),()>. Replace (:wat::kernel::join h) and (:wat::kernel::join-result h) with (:wat::kernel::Thread/join-result thr) returning :wat::core::Result<:(),:wat::core::Vector<wat::kernel::ThreadDiedError>>; match arms ((Ok _) ...) ((Err chain) ...). Mini-TCP workers (docs/ZERO-MUTEX.md) close over caller-held channels; substrate-allocated `_in` / `_out` stay unused. Workers not fitting :Fn(:Receiver<I>, :Sender<O>) -> :() — non-channel sig, non-unit return, R-via-join ferrying — get a `;; ARC 114 MANUAL — needs type-design review` comment and skip; judgment calls don't auto-sweep."}] :span {:file "test.wat" :line 1 :col 1}}"#,
        "TypeMismatch arc114 shape must emit exact structured :remedies Vector (NO :hint)"
    );

    let items = assert_remedies_vector(&edn);
    assert!(!items.is_empty(), ":remedies must be non-empty for arc 114 shape mismatch");

    let first_str = wat_edn::write(&items[0]);
    assert_eq!(
        first_str,
        r#"#wat.kernel/Remedy {:form ":wat::kernel::spawn-thread" :kind :retirement :score 0 :note "arc 114 — :wat::kernel::spawn / :wat::kernel::join / :wat::kernel::join-result retire. Programs deliver values only via their output channel; R-via-join is gone. Migrate: (:wat::kernel::spawn :worker args...) → (:wat::kernel::spawn-thread (:wat::core::fn ((_in :rust::crossbeam_channel::Receiver<()>) (_out :rust::crossbeam_channel::Sender<()>)) (:worker args...))) returning :wat::kernel::Thread<(),()>. Replace (:wat::kernel::join h) and (:wat::kernel::join-result h) with (:wat::kernel::Thread/join-result thr) returning :wat::core::Result<:(),:wat::core::Vector<wat::kernel::ThreadDiedError>>; match arms ((Ok _) ...) ((Err chain) ...). Mini-TCP workers (docs/ZERO-MUTEX.md) close over caller-held channels; substrate-allocated `_in` / `_out` stay unused. Workers not fitting :Fn(:Receiver<I>, :Sender<O>) -> :() — non-channel sig, non-unit return, R-via-join ferrying — get a `;; ARC 114 MANUAL — needs type-design review` comment and skip; judgment calls don't auto-sweep."}"#,
        "arc 114 remedy must be exact #wat.kernel/Remedy with spawn-thread :form"
    );

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 3 — ReturnTypeMismatch on retired callee: :remedies + no :hint ────

#[test]
fn probe_3_return_type_mismatch_retired_callee_emits_remedies_not_hint() {
    // ReturnTypeMismatch with no stored remedies; the retirement lookup fires
    // on the function name ":wat::core::list" (retired). Per
    // DESIGN-296-remediation-collapse line 32, the serializer MERGES the stored
    // `remedies` field with `type_error_remedies(function, expected, got)` — so
    // an empty stored field still surfaces the retirement suggestion. The arc 296
    // derive preserves this via `return_type_remedies_via` (variant-level `via`).
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

    // Must be exact EDN: :remedies with retirement entry for :wat::core::list; no :hint.
    assert_eq!(
        s,
        r#"#wat.check/ReturnTypeMismatch {:function ":wat::core::list" :expected ":wat::core::Vector<:wat::core::i64>" :got ":wat::core::nil" :remedies [#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "rename `:wat::core::list` → `:wat::core::Vector` (was a duplicate of vec; arc 109 slice 1g); substrate produces the same Vec<T> value"}] :span {:file "test.wat" :line 1 :col 1}}"#,
        "ReturnTypeMismatch on retired list must emit structured :remedies (NO :hint)"
    );

    let items = assert_remedies_vector(&edn);
    assert!(!items.is_empty(), ":remedies must be non-empty for retired function :wat::core::list");

    let first_str = wat_edn::write(&items[0]);
    assert_eq!(
        first_str,
        r#"#wat.kernel/Remedy {:form ":wat::core::Vector" :kind :retirement :score 0 :note "rename `:wat::core::list` → `:wat::core::Vector` (was a duplicate of vec; arc 109 slice 1g); substrate produces the same Vec<T> value"}"#,
        "list retirement remedy must be exact #wat.kernel/Remedy with :kind :retirement"
    );

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}
