//! `wat::panic_hook` — Rust-styled failure output for wat tests.
//!
//! Arc 016 slice 3. Replaces the old
//! `install_silent_assertion_panic_hook` (which silently swallowed
//! `AssertionPayload` panics, leaving the test runner to render a
//! bare failure message) with a hook that prints **Rust-styled**
//! failure output populated with **wat-level** content:
//!
//! ```text
//! thread 'wat test' panicked at wat-tests/LocalCache.wat:12:5:
//! assert-eq failed
//!   actual:   -1
//!   expected: 42
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! The format mirrors `cargo test`'s own failure output line-for-
//! line. Users running `cargo test` don't context-switch — same
//! phrasing, same `note:` hint, same `stack backtrace:` block under
//! `RUST_BACKTRACE=1`.
//!
//! # Design
//!
//! - **Wat-level `file:line:col`**, not Rust-level. The hook reads
//!   `AssertionPayload.location` populated by
//!   [`crate::runtime::snapshot_call_stack`] at panic time (arc 016
//!   slice 2). Rust-level panic locations (`src/runtime.rs:891`)
//!   never surface to the user.
//!
//! - **Frames are wat call-stack frames**, not Rust backtrace frames.
//!   Each frame is `<callee_path> at <file>:<line>`. Matches what
//!   the user reads in their own source.
//!
//! - **`RUST_BACKTRACE=1` gates frame rendering.** One env variable
//!   the user already knows. Cached at hook install via `OnceLock`
//!   — one lookup per process, not per panic.
//!
//! - **Non-assertion panics fall through** to the previous hook
//!   (typically Rust's default). Plain `panic!("...")` from a wat
//!   primitive or a Rust-level bug still renders normally.
//!
//! # Install sites
//!
//! Called from:
//! - [`crate::compose_and_run`] — consumer binary entry.
//! - [`crate::test_runner::run_tests_from_dir`] — `cargo test` entry
//!   via `wat::test!`.
//! - `src/bin/wat.rs::main` — the `wat` CLI.
//!
//! Idempotent: calling `install` twice chains two of these hooks;
//! the inner one still fires because the outer one delegates to
//! `previous(info)` for non-AssertionPayload panics.

use crate::assertion::AssertionPayload;
use crate::runtime::FrameInfo;
use crate::span::Span;
use std::io::Write;
use std::sync::OnceLock;

/// Cached `RUST_BACKTRACE` env lookup — one env read per process,
/// checked on every failure. `true` when set to any non-"0" value
/// (matches Rust's own convention for this env var).
static RUST_BACKTRACE_ENABLED: OnceLock<bool> = OnceLock::new();

fn backtrace_enabled() -> bool {
    *RUST_BACKTRACE_ENABLED.get_or_init(|| match std::env::var("RUST_BACKTRACE") {
        Ok(v) => v != "0" && !v.is_empty(),
        Err(_) => false,
    })
}

/// Install the wat panic hook. Writes Rust-styled failure output
/// for [`AssertionPayload`] panics; passes through to the previous
/// hook for anything else.
///
/// Idempotent in the sense that repeated installs stack — each
/// new hook delegates to the prior one for non-assertion payloads.
pub fn install() {
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

/// Render an [`AssertionPayload`] as Rust-styled failure text on
/// stderr.
fn render_assertion_failure(payload: &AssertionPayload) {
    let mut out = Vec::new();
    write_assertion_failure(&mut out, payload);
    // Ignore write errors — stderr failure has no recovery path.
    let _ = std::io::stderr().write_all(&out);
}

/// Build the failure text. Separated from rendering so tests can
/// inspect the exact bytes produced.
fn write_assertion_failure<W: Write>(out: &mut W, payload: &AssertionPayload) {
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");

    // Line 1 — thread + file:line:col header.
    match &payload.location {
        Some(span) if !span.is_unknown() => {
            let _ = writeln!(
                out,
                "thread '{}' panicked at {}:",
                thread_name,
                render_span(span)
            );
        }
        _ => {
            // Fall-back when we have no location — still say we panicked.
            let _ = writeln!(out, "thread '{}' panicked:", thread_name);
        }
    }

    // Line 2 — the assertion message (+ values when present).
    let _ = writeln!(out, "{}", payload.message);
    if let Some(a) = &payload.actual {
        let _ = writeln!(out, "  actual:   {}", a);
    }
    if let Some(e) = &payload.expected {
        let _ = writeln!(out, "  expected: {}", e);
    }

    // Line 3 — note: or stack backtrace:
    if backtrace_enabled() {
        if !payload.frames.is_empty() {
            let _ = writeln!(out, "stack backtrace:");
            render_frames(out, &payload.frames);
        }
    } else {
        let _ = writeln!(
            out,
            "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"
        );
    }
}

/// Render a span as `file:line:col` — the Rust-standard format.
fn render_span(span: &Span) -> String {
    format!("{}:{}:{}", span.file, span.line, span.col)
}

/// Render the frames in `cargo test`'s `stack backtrace:` format —
/// one line per frame, numbered from 0, with the callee path + span.
fn render_frames<W: Write>(out: &mut W, frames: &[FrameInfo]) {
    for (i, frame) in frames.iter().enumerate() {
        let _ = writeln!(
            out,
            "   {}: {} at {}",
            i,
            frame.callee_path,
            render_span(&frame.call_span)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn mk_span(file: &str, line: i64, col: i64) -> Span {
        Span::new(Arc::new(file.to_string()), line, col)
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
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("wat-tests/foo.wat:12:5"), "has location: {}", s);
        assert!(s.contains("assert-eq failed"), "has message: {}", s);
        assert!(s.contains("actual:   -1"), "has actual: {}", s);
        assert!(s.contains("expected: 42"), "has expected: {}", s);
        // RUST_BACKTRACE unset by default in tests; note: line shows.
        // (We can't set env in a cached OnceLock test reliably, so just
        // assert the note happens when backtrace is off.)
        if !backtrace_enabled() {
            assert!(s.contains("note: run with"), "has note: {}", s);
        }
    }

    #[test]
    fn renders_message_only_when_location_missing() {
        let payload = AssertionPayload {
            message: "plain panic".into(),
            actual: None,
            expected: None,
            location: None,
            frames: Vec::new(),
        };
        let mut out = Vec::new();
        write_assertion_failure(&mut out, &payload);
        let s = String::from_utf8(out).unwrap();
        assert!(s.starts_with("thread "), "starts with thread: {}", s);
        assert!(s.contains("plain panic"), "has message: {}", s);
        // No `at FILE:` when location is None.
        assert!(!s.contains(" at <synthetic>:"), "no synthetic location: {}", s);
    }
}
