//! `wat::panic_hook` — EDN-structured failure output for an unhandled assertion.
//!
//! Arc 016 slice 3 (text format). Arc 211b (EDN format). Excursus 003 strike A
//! (F1, "one death shape") — retired the hand-built `#wat.kernel/AssertionFailure
//! {…}` envelope (a SECOND top-level death shape for the same event every other
//! death reports through `LociDiedError.Panic`, and one whose writer disagreed
//! with its own declaration's `Option` convention — 0 wat readers, per
//! AUDIT-the-shape-of-an-error.md F1). This hook now renders the SAME chain shape
//! every death uses:
//!
//! ```text
//! [#wat.kernel/LociDiedError.Panic
//!  {:failure #wat.kernel/Failure
//!    {:error #wat.runtime/AssertionFailed
//!      {:message "assert-eq failed" :location {…}
//!       :actual "-1" :expected "42"}
//!     :frames […]
//!     :frames-elided 0}}]
//! ```
//!
//! Built via `crate::kernel::error::thread_crash_panic_edn` — the SAME builder the
//! thread-tier spawn machinery already used for a peer's crash-channel reason
//! (`src/kernel/spawn.rs`); this hook is now just another caller of it, not a
//! second hand-rolled writer. `types: None` is safe here: every value in this
//! tree (`LociDiedError`, `Failure`, `AssertionFailed`, `Frame`, `Span`) is a
//! self-describing builtin whose field names come from a build-time-cached wat
//! declaration, never a live `TypeEnv` lookup — see `record_field_by_name`'s own
//! callers for the one case that DOES need one (none of them are here).
//!
//! # Why this hook, and not only the explicit chain-emitters
//!
//! A peer that dies from an unhandled assertion is reported on stderr from
//! exactly ONE place, for all three ways a peer can die (excursus 003 strike A's
//! own "measure first"):
//!   - **the main thread of a top-level `wat` run** — this hook is the only
//!     stderr writer for the case nothing else catches the unwind at all;
//!   - **a fork/`spawn-program` child** — `src/process/verbs.rs`'s
//!     `finish_forked_child` no longer ALSO writes an assertion-branch line
//!     (it used to, alongside this hook, which is exactly the "two death
//!     shapes" F1 found — `emit_panics_to_stderr` is now called only for a
//!     plain, non-`AssertionPayload` panic);
//!   - **a thread peer** — `src/kernel/spawn.rs`'s spawn machinery sends the
//!     crash reason to the parent over `crash_tx` (never stderr); THIS hook is
//!     the only stderr writer for that death too, since the hook is
//!     process-global and fires on every thread's panic.
//!
//! So this hook — not a per-locus emitter — is the ONE place responsible for the
//! stderr line, and every locus's downstream code gets out of its way instead of
//! writing a second, redundant line.
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
//! Legacy explicit call sites (run_program, test_runner, runtime,
//! wat-cli) remain in place as idempotent no-ops; they may be cleaned
//! up in a later sweep.
//!
//! Idempotent: first call installs; repeated calls are no-ops
//! (guarded by `INSTALLED: AtomicBool`).  Arc 211a.

use crate::assertion::AssertionPayload;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Render an [`AssertionPayload`] as the one canonical chain-shape EDN line on
/// stderr: `[#wat.kernel/LociDiedError.Panic {:failure #wat.kernel/Failure {…}}]`.
///
/// Excursus 003 strike A (F1) — was `write_assertion_failure` (the retired
/// `#wat.kernel/AssertionFailure {…}` hand-built envelope); now delegates to
/// [`crate::kernel::error::thread_crash_panic_edn`], the SAME builder the
/// thread-tier spawn machinery uses for a crashed peer's crash-channel reason
/// (`src/kernel/spawn.rs`) — one renderer, not two. `types: None`: every value
/// in this tree is a self-describing builtin (see the module doc).
///
/// `payload.clone()` is unavoidable here: `thread_crash_panic_edn` takes an
/// owned `AssertionPayload` (it is ALSO called from contexts that only have one
/// to give, e.g. after `extract_panic_payload` consumes a `Box<dyn Any>`), and
/// the hook itself only ever borrows `payload` from `info.payload()`.
fn render_assertion_failure(payload: &AssertionPayload) {
    let line = assertion_panic_chain_line(payload);
    // Ignore write errors — stderr failure has no recovery path.
    let _ = std::io::stderr().write_all(line.as_bytes());
}

/// Build the line [`render_assertion_failure`] writes, without touching stderr —
/// separated so tests can inspect the exact bytes produced (mirrors the retired
/// `write_assertion_failure`'s testability, one layer up: a String, not a `Write`
/// sink, since there is no longer any byte-level assembly happening here to test).
fn assertion_panic_chain_line(payload: &AssertionPayload) -> String {
    format!(
        "{}\n",
        crate::kernel::error::thread_crash_panic_edn(
            payload.message.clone(),
            Some(payload.clone()),
            None,
        )
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;
    use std::sync::Arc;
    use wat_edn::OwnedValue;

    fn mk_span(file: &str, line: i64, col: i64) -> Span {
        Span::new(Arc::new(file.to_string()), line, col)
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

    fn as_map(v: &OwnedValue) -> &[(OwnedValue, OwnedValue)] {
        v.as_map().unwrap_or_else(|| panic!("expected a map, got {:?}", v))
    }

    /// Unwrap a `#wat.core/Option.Some {:value X}` / `#wat.core/Option.None {}`
    /// tagged value into a Rust `Option<&OwnedValue>` — the human-facing tagged-
    /// None convention every `Option` field in this tree uses (never a bare `nil`).
    fn option_value(v: &OwnedValue) -> Option<&OwnedValue> {
        let (tag, body) = v.as_tagged().unwrap_or_else(|| panic!("expected a tagged Option, got {:?}", v));
        match tag.to_string().as_str() {
            "#wat.core/Option.Some" => Some(get_field(as_map(body), "value")),
            "#wat.core/Option.None" => None,
            other => panic!("expected Option.Some/Option.None, got {:?}", other),
        }
    }

    /// Parse [`assertion_panic_chain_line`]'s output as the one canonical chain
    /// shape — `[#wat.kernel/LociDiedError.Panic {:failure #wat.kernel/Failure
    /// {...}}]` — and return the `Failure` map's pairs. Every payload in this
    /// module has `upstream_chain: None`, so the chain is always a singleton;
    /// asserting that here is itself part of the "one death shape" claim (F1).
    fn parse_failure(bytes: &str) -> Vec<(OwnedValue, OwnedValue)> {
        let val = wat_edn::parse_owned(bytes.trim()).expect("valid edn");
        let items = match val {
            OwnedValue::Vector(items) => items,
            other => panic!("expected a bare Vector<LociDiedError> chain, got {:?}", other),
        };
        assert_eq!(items.len(), 1, "expected a singleton chain (no upstream): {:?}", items);
        let (tag, body) = items[0]
            .as_tagged()
            .unwrap_or_else(|| panic!("expected a tagged LociDiedError, got {:?}", items[0]));
        assert_eq!(
            format!("{}/{}", tag.namespace(), tag.name()),
            "wat.kernel/LociDiedError.Panic",
            "an unhandled assertion is ALWAYS a Panic, never a second top-level shape (F1)"
        );
        let failure = get_field(as_map(body), "failure");
        let (ftag, fbody) = failure
            .as_tagged()
            .unwrap_or_else(|| panic!("expected a tagged Failure, got {:?}", failure));
        assert_eq!(ftag.to_string(), "#wat.kernel/Failure", "failure tag: {:?}", ftag);
        as_map(fbody).to_vec()
    }

    #[test]
    fn renders_location_and_values_when_present() {
        let payload = AssertionPayload {
            message: "assert-eq failed".into(),
            actual: Some("-1".into()),
            expected: Some("42".into()),
            location: Some(mk_span("wat-tests/foo.wat", 12, 5)),
            frames: vec![crate::value::frame::Frame::from(crate::value::FrameInfo::pristine(
                ":my::app::foo".into(),
                mk_span("wat-tests/foo.wat", 12, 5),
            ))],
            upstream_chain: None,
            thread_name: Some("wat-test::my-deftest".into()),
            raised_error: None,
        };
        let out = assertion_panic_chain_line(&payload);
        let failure_pairs = parse_failure(&out);

        // :error is the assertion's OWN record (excursus 003 F2) — a
        // `#wat.runtime/AssertionFailed`, never a plain `#wat.core/Fault` and
        // never the retired `#wat.kernel/AssertionFailure`.
        let error = get_field(&failure_pairs, "error");
        let (error_tag, error_body) = error.as_tagged().expect("error is a tagged record");
        assert_eq!(error_tag.to_string(), "#wat.runtime/AssertionFailed", "error tag: {:?}", error_tag);
        let error_pairs = as_map(error_body);

        // :message
        let msg = get_field(error_pairs, "message");
        assert_eq!(msg.as_str(), Some("assert-eq failed"), "message: {:?}", msg);

        // :location is a #wat.core/Span tagged record — mandatory on AssertionFailed
        // (it is a floor `:wat::core::Error`, whose `location` is never Optional).
        let loc = get_field(error_pairs, "location");
        let (loc_tag, loc_body) = loc.as_tagged().expect("location is a tagged Span");
        assert_eq!(loc_tag.to_string(), "#wat.core/Span", "location tag: {:?}", loc_tag);
        let loc_pairs = as_map(loc_body);
        let file_val = get_field(loc_pairs, "file");
        assert_eq!(file_val.as_str(), Some("wat-tests/foo.wat"), "file: {:?}", file_val);
        let line_val = get_field(loc_pairs, "line");
        assert_eq!(line_val.as_i64(), Some(12), "line: {:?}", line_val);
        let col_val = get_field(loc_pairs, "col");
        assert_eq!(col_val.as_i64(), Some(5), "col: {:?}", col_val);
        // :end is `#wat.core/Option.None {}` for `mk_span`'s point-span (no end
        // line/col supplied) — `Span: ToEdn`'s own `Option<Pos>` encoding, never a
        // bare `nil` (arc 296's "absence spoken as a tagged None", derive_tests.rs).
        let end_val = get_field(loc_pairs, "end");
        assert_eq!(option_value(end_val), None, "end should be Option.None: {:?}", end_val);

        // :actual and :expected are AssertionFailed's OWN fields now (F2) — `Failure`
        // itself carries neither.
        let actual = get_field(error_pairs, "actual");
        assert_eq!(option_value(actual).and_then(|v| v.as_str()), Some("-1"), "actual: {:?}", actual);
        let expected = get_field(error_pairs, "expected");
        assert_eq!(option_value(expected).and_then(|v| v.as_str()), Some("42"), "expected: {:?}", expected);

        // Failure/frames-elided is honestly 0 (an AssertionPayload's frames are
        // always the full, uncapped snapshot).
        let elided = get_field(&failure_pairs, "frames-elided");
        assert_eq!(elided.as_i64(), Some(0), "frames-elided: {:?}", elided);
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
            raised_error: None,
        };
        let out = assertion_panic_chain_line(&payload);
        let failure_pairs = parse_failure(&out);

        let error = get_field(&failure_pairs, "error");
        let (error_tag, error_body) = error.as_tagged().expect("error is a tagged record");
        assert_eq!(error_tag.to_string(), "#wat.runtime/AssertionFailed", "error tag: {:?}", error_tag);
        let error_pairs = as_map(error_body);

        // :message
        let msg = get_field(error_pairs, "message");
        assert_eq!(msg.as_str(), Some("plain panic"), "message: {:?}", msg);

        // No frame was captured, so no user span was available: excursus 003's
        // `location: None` answer (BRIEF-shape-strike-A) — `#[track_caller]` on
        // `failure_value_from_assertion_payload` supplies its OWN caller's Rust
        // site as the fallback (`thread_died_error_panic`'s call to it, inside
        // `src/kernel/error.rs`), never a bare/missing location (the floor's
        // `location` is mandatory).
        let loc = get_field(error_pairs, "location");
        let (loc_tag, loc_body) = loc.as_tagged().expect("location is a tagged Span even in the no-frame case");
        assert_eq!(loc_tag.to_string(), "#wat.core/Span", "location tag: {:?}", loc_tag);
        let file_val = get_field(as_map(loc_body), "file");
        assert_eq!(file_val.as_str(), Some("src/kernel/error.rs"), "no-frame fallback names the Rust call site that builds the Failure: {:?}", file_val);

        // :actual nil, :expected nil — Option.None, not a bare nil.
        let actual = get_field(error_pairs, "actual");
        assert_eq!(option_value(actual), None, "actual should be Option.None: {:?}", actual);
        let expected = get_field(error_pairs, "expected");
        assert_eq!(option_value(expected), None, "expected should be Option.None: {:?}", expected);
    }
}
