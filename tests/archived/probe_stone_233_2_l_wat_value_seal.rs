//! FM 2-bis probe for arc 233 Stone 233.2.l (#[wat_value] proc-macro structural seal).
//!
//! Asserts that the proc-macro exists, applies cleanly to container-variant
//! enums, and (transitively via cargo build) applies to the real `pub enum Value`
//! in src/runtime.rs.
//!
//! Pre-stone state:
//!   - Probe 1 FAILS to compile (`wat_macros::wat_value` doesn't exist; can't import)
//!   - Probe 2 FAILS to compile (same import failure)
//!   - Probe 3 PASSES (smoke test — cargo build succeeds with current Value enum,
//!     which doesn't have the macro applied yet; this stays green throughout)
//!
//! Post-stone state: all 3 PASS.
//!
//! **Compile-fail contracts (NOT in this probe; implemented as trybuild fixtures
//! by sonnet):** rejecting wrapping variants (sub-DESIGN contract 1); accepting
//! escape-hatch opt-in (contract 3); adversarial alias bypass (contract 5).
//! Those live under `crates/wat-macros/tests/ui/*.rs` with `*.stderr` snapshots
//! per the trybuild convention. Their assertion mechanism is structurally different
//! from runtime tests (they verify rustc REJECTS bad input).
//!
//! Stays as permanent regression guard. Per FAILURE-ENGINEERING.md ✅✅✅:
//! the proc-macro makes wrapping-variant re-introduction COMPILE-ERROR; this
//! probe ensures the macro itself exists + applies cleanly + the real Value
//! enum is sealed.

#[allow(unused_imports)]
use wat::runtime::Value;

// ─── Probe 1 — wat_macros::wat_value exists and applies to container enum ───

#[test]
fn probe_1_wat_value_applies_to_container_only_enum() {
    // Stone 233.2.l mints #[wat_value] in wat-macros/src/wat_value.rs and
    // exports it via wat-macros/src/lib.rs as `pub use wat_value::wat_value`.
    // Pre-stone: the import path doesn't exist; compile FAILS.
    use wat_macros::wat_value;

    // Container-only enum should pass the structural seal (no Box<Self> /
    // Arc<Self> / Rc<Self> / Self fields).
    #[wat_value]
    #[allow(dead_code)]
    enum SafeContainerEnum {
        Leaf(i64),
        VecOfSelf(Vec<SafeContainerEnum>),
        OptionOfSelf(Option<Box<SafeContainerEnum>>),
        // Note: Option<Box<Self>> — Box is INSIDE Option (container), not a
        // direct wrapping field. Allowed because match dispatches on the
        // Option variant, not the inner Self.
    }

    // Smoke check: enum is constructable and matchable post-macro-application.
    let v = SafeContainerEnum::Leaf(42);
    assert!(matches!(v, SafeContainerEnum::Leaf(42)));
}

// ─── Probe 2 — Escape-hatch opt-in syntax parses (smoke) ────────────────────
//
// The macro accepts `#[wat_value(allow_wrapping = "reason")]` per-variant
// for explicit opt-in. This probe doesn't verify the macro REJECTS-without-opt-in
// (that's trybuild's job — compile-fail fixture). It verifies the OPT-IN syntax
// is accepted and yields a usable enum.

#[test]
fn probe_2_wat_value_accepts_opt_in_escape_hatch() {
    use wat_macros::wat_value;

    #[wat_value]
    #[allow(dead_code)]
    enum LegacyInteropEnum {
        Leaf(i64),

        // Hypothetical legitimate use case (we have NONE today; this is
        // demonstration only). The reason string is mandatory + non-empty.
        #[wat_value(allow_wrapping = "demo only — no real use case in arc 233")]
        Wrapper { inner: Box<LegacyInteropEnum> },
    }

    let v = LegacyInteropEnum::Leaf(1);
    assert!(matches!(v, LegacyInteropEnum::Leaf(1)));

    let w = LegacyInteropEnum::Wrapper {
        inner: Box::new(LegacyInteropEnum::Leaf(2)),
    };
    assert!(matches!(w, LegacyInteropEnum::Wrapper { .. }));
}

// ─── Probe 3 — Real Value enum is reachable (smoke; passes pre + post) ──────
//
// This probe stays green throughout — its purpose is to assert that the
// wat crate continues to compile + Value enum continues to be reachable
// from external tests. Post-stone, the #[wat_value] macro is applied to
// the real Value enum in src/runtime.rs; this probe being green means
// the macro's application didn't break Value's usability.

#[test]
fn probe_3_value_enum_constructable() {
    let v = Value::i64(42);
    let _: Value = v;
    // Value enum must remain constructable + matchable for cascade tests
    // to keep functioning.
}
