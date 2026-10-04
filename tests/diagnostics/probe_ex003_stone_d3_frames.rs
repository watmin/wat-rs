//! Excursus 003 D3/strike D probe — "every runtime error carries its frames: wat frames
//! and one Rust frame" (BRIEF-envelope-step-2-every-error-carries-its-frames.md, "Prove
//! it" rows), reshaped to `{fn at tail-elided}` (BRIEF-shape-strike-D).
//!
//! `RuntimeError::new` now snapshots the live wat `CALL_STACK` (capped, correctly
//! reconstructed) plus the ONE Rust frame naming the constructing site
//! (`#[track_caller]`) AND the activation currently dispatching
//! (`current_activation`), and wires both into `:frames` / `:frames-elided` on
//! `RuntimeError::to_edn()` — the Rust frame FIRST (its true innermost position), then
//! the wat frames innermost-first. `AssertionPayload` (assertion failures) captures
//! through the same `Frame` shape, never a Rust frame of its own.
//!
//! WAT fixture: tests/diagnostics/probe_ex003_stone_d3_frames.wat

use wat::edn::contract::ToEdn;
use wat::freeze::{call_beside, call_beside_value, DeftestOutcome};
use wat_edn::OwnedValue;

fn get_field<'a>(pairs: &'a [(OwnedValue, OwnedValue)], key: &str) -> &'a OwnedValue {
    for (k, v) in pairs {
        if let Some(kw) = k.as_keyword() {
            if kw.name() == key && kw.namespace().is_none() {
                return v;
            }
        }
    }
    panic!("key :{key} not found in map {pairs:?}");
}

fn frame_fields(frame: &OwnedValue) -> &[(OwnedValue, OwnedValue)] {
    let (tag, body) = frame.as_tagged().expect("frame is a tagged Frame");
    assert_eq!(tag.namespace(), "wat.kernel", "frame tag namespace: {tag}");
    assert_eq!(tag.name(), "Frame", "frame tag name: {tag}");
    body.as_map().expect("Frame body is a map")
}

fn frame_fn(frame: &OwnedValue) -> String {
    get_field(frame_fields(frame), "fn").as_str().expect(":fn is a String").to_string()
}

fn frame_at_has_end(frame: &OwnedValue) -> bool {
    let at = get_field(frame_fields(frame), "at");
    let (_, span_body) = at.as_tagged().expect("at is a tagged Span");
    let span_pairs = span_body.as_map().expect("Span body is a map");
    let end = get_field(span_pairs, "end");
    let (end_tag, _) = end.as_tagged().expect("end is a tagged Option");
    end_tag.namespace() == "wat.core" && end_tag.name() == "Option.Some"
}

/// (a) A runtime error raised three user-calls deep shows, innermost first: the ONE Rust
/// frame (its true innermost position, excursus 003 strike D item 2), then three wat
/// frames for `inner`/`middle`/`outer` — EVERY wat frame's `at` is a genuine wat span
/// (carries a real `end`): none of them read `outer`'s own entry edge (the Rust test
/// harness's `call_beside_value` → `apply_function(.., rust_caller_span!())` call site),
/// because strike D's shift logic never surfaces a slot's OWN entry edge as anyone's
/// `at` — "the Rust call site of a top-level fn disappears, because it is the `at` of no
/// wat function" (BRIEF-shape-strike-D item 2). Only the Rust frame (frames[0]) lacks a
/// real `end`.
#[test]
fn three_deep_raise_shows_three_wat_frames_plus_one_rust_frame() {
    let err = call_beside_value(file!(), ":user::outer")
        .expect_err(":user::outer must raise (division by zero, three calls deep)");
    let edn = err.to_edn();
    let (_, body) = edn.as_tagged().expect("RuntimeError EDN is tagged");
    let pairs = body.as_map().expect("RuntimeError EDN body is a map");
    let frames = get_field(pairs, "frames").as_vector().expect(":frames is a vector");
    let elided = get_field(pairs, "frames-elided").as_i64().expect(":frames-elided is an int");

    assert_eq!(elided, 0, "a 3-deep stack fits under the cap — nothing elided");
    assert_eq!(frames.len(), 4, "1 Rust frame + 3 wat frames (inner/middle/outer)");

    assert!(
        !frame_at_has_end(&frames[0]),
        "the Rust frame's `at` is a #[track_caller] point-span, never a real `end`: {:?}",
        frames[0]
    );
    for f in &frames[1..4] {
        assert!(
            frame_at_has_end(f),
            "every wat frame's `at` must be a genuine wat-authored span (strike D never \
             surfaces a slot's own Rust-originated entry edge): {f:?}"
        );
    }
    assert_eq!(frame_fn(&frames[1]), ":user::inner");
    assert_eq!(frame_fn(&frames[2]), ":user::middle");
    assert_eq!(frame_fn(&frames[3]), ":user::outer");
}

/// (b) A tail-recursive loop that errors at depth 1,000,000 shows a small frame list — the
/// stack stayed flat (`replace_top_frame`), not 1,000,000 deep.
#[test]
fn tail_recursion_to_a_million_stays_flat() {
    let err = call_beside_value(file!(), ":user::tail-loop-entry")
        .expect_err(":user::tail-loop-entry must raise at n=0");
    let edn = err.to_edn();
    let (_, body) = edn.as_tagged().expect("RuntimeError EDN is tagged");
    let pairs = body.as_map().expect("RuntimeError EDN body is a map");
    let frames = get_field(pairs, "frames").as_vector().expect(":frames is a vector");
    assert!(
        frames.len() < 100,
        "tail recursion must keep CALL_STACK flat regardless of iteration count; \
         got {} frames — the trampoline's replace_top_frame stopped working",
        frames.len()
    );
}

/// (c) A non-tail recursion that errors deep (5,000 frames, wrapped so the recursive call is
/// never in tail position) shows the CAPPED shape with a nonzero elided count.
#[test]
fn non_tail_recursion_deep_shows_capped_shape_with_elided_count() {
    let err = call_beside_value(file!(), ":user::non-tail-loop-entry")
        .expect_err(":user::non-tail-loop-entry must raise at n=0");
    let edn = err.to_edn();
    let (_, body) = edn.as_tagged().expect("RuntimeError EDN is tagged");
    let pairs = body.as_map().expect("RuntimeError EDN body is a map");
    let frames = get_field(pairs, "frames").as_vector().expect(":frames is a vector");
    let elided = get_field(pairs, "frames-elided").as_i64().expect(":frames-elided is an int");

    // 40 (32 innermost + 8 outermost) :Wat frames + 1 :Rust frame — see
    // FRAME_CAP_INNERMOST/FRAME_CAP_OUTERMOST, src/value/frame.rs.
    assert_eq!(frames.len(), 41, "cap is innermost 32 + outermost 8 + 1 :Rust frame");
    assert!(
        elided > 200,
        "300-deep non-tail recursion must elide most of the middle; got {elided}"
    );
}

/// (d) An assertion failure's `:frames` uses the new `Frame` shape (`:fn`/`:at`/
/// `:tail-elided`), the same shape `RuntimeError`'s frames use — never a Rust frame (an
/// assertion fires from wat; `AssertionPayload` carries no `rust_frame` of its own).
#[test]
fn assertion_failure_frames_use_the_new_frame_shape() {
    let outcome = call_beside(file!(), ":user::assert-deep");
    let failure = match outcome {
        DeftestOutcome::Failed { failure } => failure,
        other => panic!("expected the deftest to FAIL (assert-eq 1 2), got {other:?}"),
    };
    let edn = wat::edn::render::value_to_edn_with(&failure, None)
        .expect("Failure value must render to EDN");
    let (_, body) = edn.as_tagged().expect("Failure EDN is tagged");
    let pairs = body.as_map().expect("Failure EDN body is a map");
    let frames = get_field(pairs, "frames").as_vector().expect(":frames is a vector");
    assert!(!frames.is_empty(), "assert-eq three calls deep must capture at least one frame");
    for f in frames {
        let fields = frame_fields(f);
        assert!(
            !frame_fn(f).is_empty(),
            "fn must be a non-empty String, never a placeholder: {f:?}"
        );
        assert!(
            get_field(fields, "at").as_tagged().is_some(),
            ":at must be a tagged Span: {f:?}"
        );
        assert!(
            get_field(fields, "tail-elided").as_i64().is_some(),
            ":tail-elided must be an int: {f:?}"
        );
    }
}
