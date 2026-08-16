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
#[test]
fn pure_enum_with_struct_field_rejected() {
    match startup_beside(file!()) {
        Ok(_) => panic!(
            "a :Pure enum declaring a struct variant field must be REJECTED by the containment rule \
             (293.W.2b); the fixture loaded cleanly — the purity wall is not enforced"
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            // CLASS-C RULING (296 Wave B1, builder overrule 2026-08-15): this golden pins an
            // INTERNAL `src/check.rs` `rust_caller_span!()` — the Rust source line:col of the
            // `TypeError::new` call site that raised `ImpureVariantFieldInPureEnum`, not a user
            // `.wat` span. The orchestrator proposed normalizing/dropping it because any edit
            // above that line in check.rs re-churns the pinned line. The builder overruled:
            // (1) the churn cost is trivial — exactly one other `.edn` golden in the tree pins a
            // `src/*.rs` span; (2) a pinned line that gets updated when it moves is in a constant
            // state of correctness, while a DROPPED field is permanently blind; (3) the span
            // DISCRIMINATES THE EMITTER — `ImpureVariantFieldInPureEnum` can be raised from more
            // than one call site in check.rs, and `rust_caller_span!()` says which. Drop it and
            // this test goes green the moment a *different* code path starts raising the same
            // error kind — that silent pass is exactly the coverage this pin buys. KEEP PINNING
            // THE SPAN. Do not re-propose dropping it.
            wat::assert_edn_matches_file!(msg, "probe_arc293_W2b_enum_purity__pure_enum_with_struct_field_rejected.edn", "Pure enum declaring a struct variant field: ImpureVariantFieldInPureEnum (internal check.rs span pinned — see comment above)");
        }
    }
}
