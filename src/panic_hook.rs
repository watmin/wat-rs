//! `wat::panic_hook` — EDN-structured failure output for wat tests.
//!
//! Arc 016 slice 3 (text format). Arc 211b (EDN format). Replaces the
//! old `install_silent_assertion_panic_hook` (which silently swallowed
//! `AssertionPayload` panics) with a hook that prints structured EDN:
//!
//! ```text
//! #wat.kernel/AssertionFailure {
//!   :thread "wat-test::my-deftest"
//!   :message "assert-eq failed"
//!   :location {:file "wat-tests/foo.wat" :line 12 :col 5}
//!   :actual "-1"
//!   :expected "42"
//!   :frames [{:callee :my.app/foo :at {:file "wat-tests/foo.wat" :line 12 :col 5}}]
//!   :upstream-chain nil
//! }
//! ```
//!
//! The format is machine-parseable EDN. Mirrors the existing
//! `#wat.kernel/ProcessPanics{...}` envelope from arc 170 slice 1i.
//! Humans read EDN just fine.
//!
//! # Design
//!
//! - **`payload_to_edn`** builds an `OwnedValue` map from an `AssertionPayload`.
//!   Called by `write_assertion_failure`.
//!
//! - **`:frames` always present** (empty vector when no frames). Consumer
//!   decides display; no env-var gating. RUST_BACKTRACE is no longer
//!   consulted (arc 211b removes the env-var gating).
//!
//! - **Non-assertion panics fall through** to the previous hook
//!   (typically Rust's default). Plain `panic!("...")` from a wat
//!   primitive or a Rust-level bug still renders normally.
//!
//! # Install sites
//!
//! Arc 211a: auto-installed at library load via `#[ctor::ctor]` —
//! fires before `main()` in every binary that links `wat`.
//! Impossible to forget by construction.
//!
//! Legacy explicit call sites (compose_and_run, test_runner, runtime,
//! wat-cli) remain in place as idempotent no-ops; they may be cleaned
//! up in a later sweep.
//!
//! Idempotent: first call installs; repeated calls are no-ops
//! (guarded by `INSTALLED: AtomicBool`).  Arc 211a.

use crate::assertion::AssertionPayload;
use crate::runtime::FrameInfo;
use crate::span::Span;
use std::borrow::Cow;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use wat_edn::{Keyword, OwnedValue};

/// Tracks whether the hook has been installed. First install wins;
/// subsequent `install()` calls become idempotent no-ops (Arc 211a).
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Auto-install at library load time via `#[ctor(unsafe)]` (Arc 211a).
/// Fires before `main()` in every binary that links `wat`. Impossible
/// to forget by construction.
///
/// `unsafe` here is the ctor 1.x spelling; the implementation is
/// safe (it only calls `install()` which is a safe Rust function).
/// ctor 1.x requires this annotation because library constructors
/// can in principle run before the Rust runtime is fully initialized.
#[ctor::ctor(unsafe)]
fn auto_install() {
    install();
}

/// Install the wat panic hook. Writes EDN-structured failure output
/// for [`AssertionPayload`] panics; passes through to the previous
/// hook for anything else.
///
/// Idempotent: first call installs; subsequent calls are no-ops.
/// The `#[ctor::ctor]` auto-install (Arc 211a) fires before `main()`;
/// explicit call sites (compose, test_runner, runtime, wat-cli) remain
/// as no-ops and may be cleaned up in a later sweep.
pub fn install() {
    // Arc 211a: first-call-wins idempotency. swap returns the OLD value;
    // if it was already true, someone beat us here — return immediately.
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(payload) = info.payload().downcast_ref::<AssertionPayload>() {
            render_assertion_failure(payload);
            return;
        }
        // Non-assertion panic — propagate to the previous hook
        // (typically Rust's default, which prints
        // "thread X panicked at src/foo.rs:L:C: <message>").
        previous(info);
    }));
}

/// Returns `true` if `install()` has completed at least once.
/// Used by the probe test to verify the `#[ctor::ctor]` auto-install
/// fired before any explicit call in test code.
pub fn is_installed() -> bool {
    INSTALLED.load(Ordering::SeqCst)
}

/// Render an [`AssertionPayload`] as EDN on stderr.
fn render_assertion_failure(payload: &AssertionPayload) {
    let mut out = Vec::new();
    write_assertion_failure(&mut out, payload);
    // Ignore write errors — stderr failure has no recovery path.
    let _ = std::io::stderr().write_all(&out);
}

/// Build the EDN failure output. Separated from rendering so tests can
/// inspect the exact bytes produced.
///
/// Arc 138 F-NAMES-1d — `thread_name` is read from the payload (captured
/// at panic site on the worker thread) rather than queried via
/// `thread::current()`. This survives `panic::resume_unwind` re-panicking
/// the payload on the parent thread.
///
/// Arc 211b — replaced text format with `#wat.kernel/AssertionFailure{...}`
/// EDN envelope mirroring `#wat.kernel/ProcessPanics{...}`.
fn write_assertion_failure<W: Write>(out: &mut W, payload: &AssertionPayload) {
    let edn_value = payload_to_edn(payload);
    let line = format!("#wat.kernel/AssertionFailure {}\n", wat_edn::write(&edn_value));
    let _ = out.write_all(line.as_bytes());
}

/// Build the `OwnedValue` map for an [`AssertionPayload`]. The map has
/// exactly 7 keys in the `#wat.kernel/AssertionFailure` envelope.
///
/// Exported as `pub(crate)` so the `mod tests` block below can call it
/// directly.
pub(crate) fn payload_to_edn(payload: &AssertionPayload) -> OwnedValue {
    // ── :thread ──────────────────────────────────────────────────────
    let thread_val = match &payload.thread_name {
        Some(name) => OwnedValue::String(Cow::Owned(name.clone())),
        None => OwnedValue::Nil,
    };

    // ── :message ─────────────────────────────────────────────────────
    let message_val = OwnedValue::String(Cow::Owned(payload.message.clone()));

    // ── :location ────────────────────────────────────────────────────
    let location_val = match &payload.location {
        Some(span) if !span.is_unknown() => span_to_map(span),
        _ => OwnedValue::Nil,
    };

    // ── :actual / :expected ──────────────────────────────────────────
    let actual_val = match &payload.actual {
        Some(a) => OwnedValue::String(Cow::Owned(a.clone())),
        None => OwnedValue::Nil,
    };
    let expected_val = match &payload.expected {
        Some(e) => OwnedValue::String(Cow::Owned(e.clone())),
        None => OwnedValue::Nil,
    };

    // ── :frames ──────────────────────────────────────────────────────
    let frames_val = OwnedValue::Vector(
        payload
            .frames
            .iter()
            .map(frame_to_map)
            .collect(),
    );

    // ── :upstream-chain ──────────────────────────────────────────────
    let chain_val = match &payload.upstream_chain {
        None => OwnedValue::Nil,
        Some(chain) if chain.is_empty() => OwnedValue::Nil,
        Some(chain) => {
            let items = chain
                .iter()
                .map(|v| crate::edn_shim::value_to_edn_with(v, None))
                .collect();
            OwnedValue::Vector(items)
        }
    };

    // ── Assemble map ─────────────────────────────────────────────────
    OwnedValue::Map(vec![
        (OwnedValue::Keyword(Keyword::new("thread")), thread_val),
        (OwnedValue::Keyword(Keyword::new("message")), message_val),
        (OwnedValue::Keyword(Keyword::new("location")), location_val),
        (OwnedValue::Keyword(Keyword::new("actual")), actual_val),
        (OwnedValue::Keyword(Keyword::new("expected")), expected_val),
        (OwnedValue::Keyword(Keyword::new("frames")), frames_val),
        (OwnedValue::Keyword(Keyword::new("upstream-chain")), chain_val),
    ])
}

/// Convert a [`Span`] to `{:file "..." :line N :col N}`.
fn span_to_map(span: &Span) -> OwnedValue {
    OwnedValue::Map(vec![
        (
            OwnedValue::Keyword(Keyword::new("file")),
            OwnedValue::String(Cow::Owned(span.file.as_str().to_owned())),
        ),
        (
            OwnedValue::Keyword(Keyword::new("line")),
            OwnedValue::Integer(span.line),
        ),
        (
            OwnedValue::Keyword(Keyword::new("col")),
            OwnedValue::Integer(span.col),
        ),
    ])
}

/// Convert a [`FrameInfo`] to `{:callee <keyword> :at <location-map>}`.
///
/// `frame.callee_path` is a string like `":my::app::foo"` (with leading `:`).
/// Strip the `:` prefix, then use `keyword_from_callee_path` to build a
/// proper EDN keyword using the same convention as `edn_shim::keyword_from_wat_path`.
fn frame_to_map(frame: &FrameInfo) -> OwnedValue {
    OwnedValue::Map(vec![
        (
            OwnedValue::Keyword(Keyword::new("callee")),
            keyword_from_callee_path(&frame.callee_path),
        ),
        (
            OwnedValue::Keyword(Keyword::new("at")),
            span_to_map(&frame.call_span),
        ),
    ])
}

/// Convert a callee path string (like `":my::app::foo"`) to an EDN keyword.
///
/// Mirrors `edn_shim::keyword_from_wat_path`: strip the leading `:`, then
/// split on the last `::` to extract namespace + name. Falls back to a
/// string if the keyword construction fails validation.
fn keyword_from_callee_path(path: &str) -> OwnedValue {
    let stripped = path.strip_prefix(':').unwrap_or(path);
    if let Some(idx) = stripped.rfind("::") {
        let ns = stripped[..idx].replace("::", ".");
        let name = &stripped[idx + 2..];
        match Keyword::try_ns(&ns, name) {
            Ok(kw) => OwnedValue::Keyword(kw),
            Err(_) => OwnedValue::String(Cow::Owned(path.to_string())),
        }
    } else {
        match Keyword::try_new(stripped) {
            Ok(kw) => OwnedValue::Keyword(kw),
            Err(_) => OwnedValue::String(Cow::Owned(path.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn mk_span(file: &str, line: i64, col: i64) -> Span {
        Span::new(Arc::new(file.to_string()), line, col)
    }

    /// Parse the written EDN bytes and return the tagged map.
    /// Returns (tag_string, map_pairs) for convenient field inspection.
    fn parse_envelope(bytes: &[u8]) -> (String, Vec<(OwnedValue, OwnedValue)>) {
        let s = std::str::from_utf8(bytes).expect("utf8");
        let val = wat_edn::parse_owned(s.trim()).expect("valid edn");
        match val {
            OwnedValue::Tagged(tag, body) => {
                let tag_str = format!("{}/{}", tag.namespace(), tag.name());
                match *body {
                    OwnedValue::Map(pairs) => (tag_str, pairs),
                    other => panic!("expected map body, got {:?}", other),
                }
            }
            other => panic!("expected tagged value, got {:?}", other),
        }
    }

    fn get_field<'a>(pairs: &'a [(OwnedValue, OwnedValue)], key: &str) -> &'a OwnedValue {
        for (k, v) in pairs {
            if let OwnedValue::Keyword(kw) = k {
                if kw.name() == key && kw.namespace().is_none() {
                    return v;
                }
            }
        }
        panic!("key :{} not found in map", key);
    }

    #[test]
    fn renders_location_and_values_when_present() {
        let payload = AssertionPayload {
            message: "assert-eq failed".into(),
            actual: Some("-1".into()),
            expected: Some("42".into()),
            location: Some(mk_span("wat-tests/foo.wat", 12, 5)),
            frames: vec![FrameInfo {
                callee_path: ":my::app::foo".into(),
                call_span: mk_span("wat-tests/foo.wat", 12, 5),
            }],
            upstream_chain: None,
            thread_name: Some("wat-test::my-deftest".into()),
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let (tag, pairs) = parse_envelope(&out);

        assert_eq!(tag, "wat.kernel/AssertionFailure", "tag: {}", tag);

        // :message
        let msg = get_field(&pairs, "message");
        assert_eq!(msg.as_str(), Some("assert-eq failed"), "message: {:?}", msg);

        // :location map with :file :line :col
        let loc = get_field(&pairs, "location");
        let loc_pairs = loc.as_map().expect("location is a map");
        let file_val = loc_pairs.iter().find(|(k, _)| k.as_keyword().map(|kw| kw.name()) == Some("file")).map(|(_, v)| v).expect(":file");
        assert_eq!(file_val.as_str(), Some("wat-tests/foo.wat"), "file: {:?}", file_val);
        let line_val = loc_pairs.iter().find(|(k, _)| k.as_keyword().map(|kw| kw.name()) == Some("line")).map(|(_, v)| v).expect(":line");
        assert_eq!(line_val.as_i64(), Some(12), "line: {:?}", line_val);
        let col_val = loc_pairs.iter().find(|(k, _)| k.as_keyword().map(|kw| kw.name()) == Some("col")).map(|(_, v)| v).expect(":col");
        assert_eq!(col_val.as_i64(), Some(5), "col: {:?}", col_val);

        // :actual and :expected
        let actual = get_field(&pairs, "actual");
        assert_eq!(actual.as_str(), Some("-1"), "actual: {:?}", actual);
        let expected = get_field(&pairs, "expected");
        assert_eq!(expected.as_str(), Some("42"), "expected: {:?}", expected);
    }

    #[test]
    fn renders_message_only_when_location_missing() {
        let payload = AssertionPayload {
            message: "plain panic".into(),
            actual: None,
            expected: None,
            location: None,
            frames: Vec::new(),
            upstream_chain: None,
            thread_name: None,
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let (tag, pairs) = parse_envelope(&out);

        assert_eq!(tag, "wat.kernel/AssertionFailure", "tag: {}", tag);

        // :message
        let msg = get_field(&pairs, "message");
        assert_eq!(msg.as_str(), Some("plain panic"), "message: {:?}", msg);

        // :location nil when absent
        let loc = get_field(&pairs, "location");
        assert_eq!(loc, &OwnedValue::Nil, "location should be nil: {:?}", loc);

        // :actual nil, :expected nil
        let actual = get_field(&pairs, "actual");
        assert_eq!(actual, &OwnedValue::Nil, "actual should be nil: {:?}", actual);
        let expected = get_field(&pairs, "expected");
        assert_eq!(expected, &OwnedValue::Nil, "expected should be nil: {:?}", expected);
    }

    // Arc 138 F-NAMES-1d — verify thread_name field is used as-is, NOT
    // queried from thread::current(). This matters because resume_unwind
    // re-panics the payload on the PARENT thread; the parent has a
    // different (or absent) name. Payload carries the worker's name.

    #[test]
    fn renders_thread_name_from_payload_field() {
        let payload = AssertionPayload {
            message: "assert-eq failed".into(),
            actual: None,
            expected: None,
            location: None,
            frames: Vec::new(),
            upstream_chain: None,
            // Explicit name that does NOT match this test's thread name.
            thread_name: Some("wat-test:::my::deftest".into()),
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let (tag, pairs) = parse_envelope(&out);

        assert_eq!(tag, "wat.kernel/AssertionFailure", "tag: {}", tag);

        let thread = get_field(&pairs, "thread");
        assert_eq!(
            thread.as_str(),
            Some("wat-test:::my::deftest"),
            "renders payload thread_name verbatim: {:?}", thread
        );
    }

    #[test]
    fn renders_unnamed_when_thread_name_field_is_none() {
        let payload = AssertionPayload {
            message: "some failure".into(),
            actual: None,
            expected: None,
            location: None,
            frames: Vec::new(),
            upstream_chain: None,
            thread_name: None,
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let (tag, pairs) = parse_envelope(&out);

        assert_eq!(tag, "wat.kernel/AssertionFailure", "tag: {}", tag);

        let thread = get_field(&pairs, "thread");
        assert_eq!(
            thread,
            &OwnedValue::Nil,
            "falls back to nil when field is None: {:?}", thread
        );
    }
}
