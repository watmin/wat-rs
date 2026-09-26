//! Excursus 003 D3 probe — "every runtime error carries its frames: wat frames and one
//! Rust frame" (BRIEF-envelope-step-2-every-error-carries-its-frames.md, "Prove it" rows).
//!
//! `RuntimeError::new` now snapshots the live wat `CALL_STACK` (capped) plus the ONE Rust
//! frame naming the constructing site (`#[track_caller]`), and wires both into `:frames` /
//! `:frames-elided` on `RuntimeError::to_edn()`. `AssertionPayload` (assertion failures)
//! captures through the same `Frame` shape.
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

/// (namespace, name) of a tagged `Frame`'s `:kind` — NOT a `#ns/Name` string (that shape
/// trips `no_inlined_edn`'s EDN-esque-literal heuristic on the caller's comparison).
fn frame_kind_tag(frame: &OwnedValue) -> (String, String) {
    let (tag, body) = frame.as_tagged().expect("frame is a tagged Frame");
    assert_eq!(tag.namespace(), "wat.kernel", "frame tag namespace: {tag}");
    assert_eq!(tag.name(), "Frame", "frame tag name: {tag}");
    let pairs = body.as_map().expect("Frame body is a map");
    let kind = get_field(pairs, "kind");
    let (kind_tag, _) = kind.as_tagged().expect("kind is a tagged FrameKind");
    (kind_tag.namespace().to_string(), kind_tag.name().to_string())
}

fn frame_span_has_end(frame: &OwnedValue) -> bool {
    let (_, body) = frame.as_tagged().expect("frame is a tagged Frame");
    let pairs = body.as_map().expect("Frame body is a map");
    let span = get_field(pairs, "span");
    let (_, span_body) = span.as_tagged().expect("span is a tagged Span");
    let span_pairs = span_body.as_map().expect("Span body is a map");
    let end = get_field(span_pairs, "end");
    let (end_tag, _) = end.as_tagged().expect("end is a tagged Option");
    end_tag.namespace() == "wat.core" && end_tag.name() == "Option.Some"
}

/// (a) A runtime error raised three user-calls deep shows three `:Wat` frames (innermost
/// first) plus one `:Rust` frame naming the constructing Rust site.
///
/// Only the two genuinely wat-to-wat call sites (`inner`'s call written inside `middle`,
/// `middle`'s call written inside `outer`) carry a real `Span` `end` — the THIRD `:Wat`
/// frame is `outer`'s OWN invocation, made by this Rust test harness
/// (`call_beside_value` → `apply_function(.., rust_caller_span!())`), so its span is
/// itself Rust-originated (`end: None`, D1's own rule) even though the frame's `kind` is
/// `:Wat` (it is a genuine `CALL_STACK` entry, not the error's constructing Rust site).
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
    assert_eq!(frames.len(), 4, "3 :Wat frames (inner/middle/outer calls) + 1 :Rust frame");

    for f in &frames[..3] {
        assert_eq!(
            frame_kind_tag(f),
            ("wat.kernel".to_string(), "FrameKind.Wat".to_string()),
            "frame: {f:?}"
        );
    }
    for f in &frames[..2] {
        assert!(
            frame_span_has_end(f),
            "a genuine wat-to-wat call site's span must carry a real end: {f:?}"
        );
    }
    assert_eq!(
        frame_kind_tag(&frames[3]),
        ("wat.kernel".to_string(), "FrameKind.Rust".to_string()),
        "the last frame must be the one :Rust frame naming the constructing site: {:?}",
        frames[3]
    );
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

/// (d) An assertion failure's `:frames` uses the new `Frame` shape (`:symbol`/`:span`/`:kind`),
/// the same shape `RuntimeError`'s frames use.
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
        assert_eq!(
            frame_kind_tag(f),
            ("wat.kernel".to_string(), "FrameKind.Wat".to_string()),
            "frame: {f:?}"
        );
        let (_, fbody) = f.as_tagged().expect("frame is tagged");
        let fpairs = fbody.as_map().expect("Frame body is a map");
        assert!(get_field(fpairs, "symbol").as_str().is_some(), "symbol must be a String");
    }
}
