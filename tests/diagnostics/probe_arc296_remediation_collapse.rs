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
use wat::edn::contract::ToEdn;
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
    wat::assert_edn_matches_file!(s.clone(), "probe_arc296_remediation_collapse__type_mismatch_vec.edn", "TypeMismatch on retired callee must emit structured :remedies Vector (NO :hint)");

    let items = assert_remedies_vector(&edn);
    assert!(!items.is_empty(), ":remedies must be non-empty for retired callee :wat::core::vec");

    let first_str = wat_edn::write(&items[0]);
    wat::assert_edn_matches_file!(first_str, "probe_arc296_remediation_collapse__vec_remedy.edn", "first remedy must be exact #wat.kernel/Remedy with :kind :retirement");

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}

// ─── Probe 2 — RETIRED 2026-08-15 (arc 296, Wave A of the recapture cascade) ──
//
// It asserted that a `ProgramHandle<T>` ↔ `Thread<R,S>` parameter mismatch emits an arc-114
// spawn-thread migration remedy. Both halves of that premise are gone:
//
//   1. The capability was DELIBERATELY DELETED — `shape_remedies` survives only as its own
//      tombstone at `src/check.rs:94` ("arc 114's shape_remedies died with the spawn/join/
//      join-result tombstones"). `retirement_lookup` matches on the erroring CALLEE's name and
//      structurally cannot express a shape-pair match.
//   2. `:wat::kernel::ProgramHandle` DOES NOT EXIST. It is not registered anywhere and appears
//      in `src/types.rs:2302` only inside a comment. The shape looked reachable — a live
//      `--check` on a `ProgramHandle<String>`-annotated parameter exits 0 — but only because an
//      annotation naming a nonexistent type is silently accepted. That silence is the DOCUMENTED,
//      RULED, PARKED flaw in `docs/arc/2026/04/109-kill-std/NOTE-type-annotation-names-unchecked.md`
//      ("a type name that does not exist is an error when it is a callee, and silence when it is
//      an annotation"), whose 2026-07-28 addendum names this exact parametric case.
//
// So the probe measured a deleted remedy for a mismatch between a nonexistent type and a live
// one. Same disposition as `probe_arc258_dotted_record_field` earlier in this arc: a test
// pinning something the substrate removed retires with it.
//
// Its two goldens (`__type_mismatch_arc114.edn`, `__arc114_remedy.edn`) are deleted with it.

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
    wat::assert_edn_matches_file!(s.clone(), "probe_arc296_remediation_collapse__return_type_mismatch_list.edn", "ReturnTypeMismatch on retired list must emit structured :remedies (NO :hint)");

    let items = assert_remedies_vector(&edn);
    assert!(!items.is_empty(), ":remedies must be non-empty for retired function :wat::core::list");

    let first_str = wat_edn::write(&items[0]);
    wat::assert_edn_matches_file!(first_str, "probe_arc296_remediation_collapse__list_remedy.edn", "list retirement remedy must be exact #wat.kernel/Remedy with :kind :retirement");

    wat_edn::parse_owned(&s).expect("must be valid EDN");
}
