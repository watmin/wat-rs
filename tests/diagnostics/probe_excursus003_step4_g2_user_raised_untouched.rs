//! Excursus 003 step 4 (D4) — G2, a user-raised error is untouched.
//!
//! See the co-located `.wat` fixture. The raise span is already the user's own line —
//! derivation must add nothing, and the exactly-once invariant (the raise span appears
//! across `:location` ∪ `:frames` exactly once) must hold trivially: it appears only as
//! `:location`, never duplicated into a synthetic frame.

use wat::edn::contract::ToEdn;
use wat::freeze::call_beside_value;

const FIXTURE_LABEL: &str = "tests/diagnostics/probe_excursus003_step4_g2_user_raised_untouched.wat";

#[test]
fn g2_a_user_raised_error_is_untouched() {
    let err = call_beside_value(file!(), ":my::test::probe-divzero")
        .expect_err("4 / 0 must raise DivisionByZero");

    // :location is exactly the user's own raise site.
    assert_eq!(err.span().file.as_str(), FIXTURE_LABEL);

    let rendered = wat_edn::write(&err.to_edn());
    let needle = format!(
        ":file \"{}\" :line {} :col {}",
        err.span().file,
        err.span().line,
        err.span().col
    );
    assert_eq!(
        rendered.matches(&needle).count(),
        1,
        "the raise span must appear EXACTLY ONCE (as :location only, never duplicated \
         into a synthetic frame); rendered: {rendered}"
    );

    // No raise frame was added: exactly one wat frame (the call to probe-divzero
    // itself, pushed by call_beside_value's own apply_function) plus the one Rust
    // frame `RuntimeError::new` always captures.
    let frames_at = rendered.find(":frames").expect(":frames key must be present");
    let after_frames = &rendered[frames_at..];
    let frame_count = after_frames.matches("#wat.kernel/Frame {").count();
    assert_eq!(
        frame_count, 2,
        "expected exactly 2 frames (the probe-divzero call site + the one Rust frame) \
         with no synthetic raise frame added; rendered: {after_frames}"
    );
}
