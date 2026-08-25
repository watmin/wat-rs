//! Disconfirming probe — arc 109 stone B (`BRIEF-runtime-error-constructor.md`).
//!
//! **The one non-trivial claim stone B rests on:** moving `RuntimeError.kind` from
//! `RuntimeErrorKind` to `Box<RuntimeErrorKind>` leaves the structured error EDN
//! **byte-identical**, so the ~1438-site sweep can never change what a caller sees.
//!
//! Grounding, so the claim is mechanism and not hope: `RuntimeError`'s `ToEdn` is
//! HAND-WRITTEN (`src/runtime_error_edn.rs:64-83`) and reads its kind through a plain
//! method call — `let kind_val = self.kind.to_edn();`. Rust auto-derefs a `Box<T>`
//! receiver, and `wat-edn` carries a blanket forwarding `impl<T: ToEdn> ToEdn for Box<T>`
//! (`crates/wat-edn/src/lib.rs:217`), so BOTH resolutions produce the same `OwnedValue`.
//! Nothing in the wrapper touches the field's storage; it re-tags and appends `:span`.
//!
//! This probe pins that empirically rather than by reading: it renders the SAME kind
//! value through both an owned and a boxed access path and asserts the two EDN strings
//! are equal. It is written to go RED if a future `ToEdn for Box<T>` ever stops being a
//! transparent forward — which is the only way stone B's sweep could silently change
//! output.
//!
//! Deliberately NOT asserted here: `size_of` (that is the width gate's job,
//! `probe_runtime_error_width.rs`) and the goldens (`probe_arc298_3_runtime_derive_identical`
//! owns those). This probe carries exactly one claim.

use wat::edn::contract::ToEdn;
use wat::value::signal::{RuntimeError, RuntimeErrorKind};

/// Build a kind with real payload — a variant that carries data, so the EDN body is
/// non-trivial and a lost/reordered field would show up as a difference.
fn sample_kind() -> RuntimeErrorKind {
    RuntimeErrorKind::EdnCoerceMismatch {
        op: "probe-op".to_string(),
        expected: Box::new("wat::core::i64".to_string()),
        got: Box::new("wat::core::String".to_string()),
        path: "field/inner".to_string(),
    }
}

#[test]
fn boxed_kind_renders_identical_edn_to_owned_kind() {
    let owned = sample_kind();
    let boxed: Box<RuntimeErrorKind> = Box::new(sample_kind());

    // The exact expression `RuntimeError::to_edn` uses on its kind field, both ways.
    let via_owned = format!("{:?}", owned.to_edn());
    let via_boxed = format!("{:?}", boxed.to_edn());

    assert_eq!(
        via_owned, via_boxed,
        "Box<RuntimeErrorKind>::to_edn() diverged from RuntimeErrorKind::to_edn(). \
         Stone B's 1438-site sweep assumes the blanket `impl<T: ToEdn> ToEdn for Box<T>` \
         is a transparent forward; if that stopped holding, boxing `kind` would silently \
         change every structured runtime error a caller sees"
    );
}

/// A second, distinct kind — so "does the wrapper read the field?" can be asked by
/// DIFFERENCE rather than by inspecting rendered text for a substring.
fn other_kind() -> RuntimeErrorKind {
    RuntimeErrorKind::EdnCoerceMismatch {
        op: "other-op".to_string(),
        expected: Box::new("wat::core::bool".to_string()),
        got: Box::new("wat::core::nil".to_string()),
        path: "different/path".to_string(),
    }
}

#[test]
fn whole_runtime_error_edn_is_reachable_through_the_kind_field() {
    // Guards the OTHER half: that `RuntimeError`'s hand-written wrapper actually reads its
    // kind THROUGH the field, so the Box-forwarding proven above sits on the real path. If
    // the wrapper is ever rewritten to bypass the field, this goes red and stone B's
    // byte-identical claim must be re-grounded rather than assumed.
    //
    // Asked by DIFFERENCE, deliberately: two errors that share a span but carry different
    // kinds must render differently. If the wrapper ignored the field, both would render
    // identically and this fails. That is an exact comparison of whole values — never a
    // substring check on a Debug string, which would pass on reordered or truncated output
    // (the `no_loose_string_assert` lint is right about that, and this probe's first draft
    // was wrong).
    let span = wat::rust_caller_span!();
    let a = format!("{:?}", RuntimeError::new(span.clone(), sample_kind()).to_edn());
    let b = format!("{:?}", RuntimeError::new(span.clone(), other_kind()).to_edn());
    assert_ne!(
        a, b,
        "two RuntimeErrors with the same span but DIFFERENT kinds rendered identical EDN, \
         so RuntimeError::to_edn() is not reading its kind through the field this stone \
         boxes — stone B's byte-identical claim no longer rests on anything"
    );

    // And the wrapper must ADD to the kind's own rendering (it re-tags and appends :span),
    // so the whole error cannot render identically to its bare kind. Also exact.
    let bare_kind = format!("{:?}", sample_kind().to_edn());
    assert_ne!(
        a, bare_kind,
        "RuntimeError::to_edn() rendered identically to its bare kind, so the wrapper is no \
         longer appending the span — its shape changed and stone B must be re-grounded"
    );
}
