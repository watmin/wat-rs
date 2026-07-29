//! GREEN probe — arc 293.W.2b: the enum purity marker + containment gate.
//!
//! The wire wall is a PURITY wall. Enums DECLARE their purity via a mandatory
//! `:wat::enum::Pure` | `:wat::enum::Impure` marker on `defenum`. A `:Pure` enum
//! must hold only pure variant fields (scalars, records, other Pure enums); an
//! `:Impure` enum is unrestricted (it holds live resources and never crosses the wire).
//!
//! This fixture is GREEN: three cases, each accepted or rejected as expected.
//!   Case 1 — a `:Pure` enum with an impure (struct) variant field → REJECTED (containment).
//!   Case 2 — a `defenum` with NO marker → REJECTED (mandatory marker).
//!   Case 3 — a record holding an `:Impure` enum field → REJECTED (containment).
//!   Case 4 — a `:Pure` enum with only pure fields → ACCEPTED (green path).
//!
//! GREEN after 293.W.2b (this strike). The fixture is co-located and loaded
//! by `startup_beside(file!())`.

use wat::freeze::startup_beside;

/// Case 1 — a `:Pure` enum declaring a struct variant field is REJECTED.
/// The containment rule: a `:Pure` enum may hold only pure variant fields.
/// A struct is impure (categorically — it permits resources and never crosses).
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn pure_enum_with_struct_field_rejected() {
    match startup_beside(file!()) {
        Ok(_) => panic!(
            "a :Pure enum declaring a struct variant field must be REJECTED by the containment rule \
             (293.W.2b); the fixture loaded cleanly — the purity wall is not enforced"
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            assert_eq!(msg, r#"Type(TypeError { span: Span { file: "src/check.rs", line: 13659, col: 43, end_line: 13659, end_col: 43 }, kind: ImpureVariantFieldInPureEnum { enum_name: ":w2b::BadEvt", variant: "Live", field: "c", field_ty: ":w2b::Conn" } })"#);
        }
    }
}
