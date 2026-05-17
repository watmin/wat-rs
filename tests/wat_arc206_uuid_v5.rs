//! Arc 206 slice 1.5 — `:wat::core::uuid::v5` substrate promotion.
//!
//! Verifies that `:wat::core::uuid::v5` is available at the substrate
//! level without any `:wat::telemetry` dep. This test file imports only
//! `wat` crate types — no `wat_telemetry`, no `wat_measure`. The mere
//! fact that these tests compile and pass is proof of substrate-level
//! availability.
//!
//! Four cases (BRIEF SCORE rows A–D):
//!   A — basic call returns a 36-char canonical-shape String
//!   B — deterministic: same (namespace, name) → same UUID
//!   C — different namespace, same name → different UUID
//!         different name, same namespace → different UUID
//!   D — workspace baseline preserved (structural; enforced by full workspace run)

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, AmbientStdio};

// RFC 4122 DNS namespace UUID — stable, well-known, suitable as a fixed test namespace.
const DNS_NAMESPACE: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader
        .read_all(wat::span::Span::unknown())
        .expect("read-all");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn run(src: &str) -> Vec<String> {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = uninstall_ambient_stdio();
    drain_lines(&stdout_capture)
}

// ─── A: basic call returns 36-char canonical String ─────────────────────────

/// `(:wat::core::uuid::v5 namespace name)` mints a deterministic UUID, prints
/// via `println`, and the captured output (after stripping surrounding `"`) is
/// exactly 36 chars in canonical hyphenated form.
#[test]
fn uuid_v5_returns_36_char_string() {
    let src = format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [id (:wat::core::uuid::v5 "{ns}" "test-name")]
            (:wat::kernel::println id)))
    "#,
        ns = DNS_NAMESPACE
    );
    let lines = run(&src);
    assert_eq!(lines.len(), 1, "expected exactly one line of output");
    // println wraps strings in quotes; strip them for shape analysis.
    let raw = lines[0].trim_matches('"');
    assert_eq!(
        raw.len(),
        36,
        "UUID must be 36 chars (8-4-4-4-12 + 4 hyphens), got {:?}",
        raw
    );
    // Verify 5-part hyphen structure.
    let parts: Vec<&str> = raw.split('-').collect();
    assert_eq!(
        parts.len(),
        5,
        "UUID must split into 5 hyphen-separated parts, got {:?}",
        raw
    );
}

// ─── B: deterministic — same (namespace, name) → same UUID ──────────────────

/// Two calls with the same namespace and name must produce identical strings.
/// A random implementation (v4) would fail this immediately.
#[test]
fn uuid_v5_deterministic_same_inputs_produce_same_uuid() {
    let src = format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a (:wat::core::uuid::v5 "{ns}" "my-resource")
             b (:wat::core::uuid::v5 "{ns}" "my-resource")]
            (:wat::core::if (:wat::core::= a b) -> :wat::core::nil
              (:wat::kernel::println "SAME")
              (:wat::kernel::println "DIFFERENT"))))
    "#,
        ns = DNS_NAMESPACE
    );
    let lines = run(&src);
    assert_eq!(
        lines,
        vec!["\"SAME\""],
        "v5 must be deterministic: same (namespace, name) → same UUID"
    );
}

// ─── C1: different namespace, same name → different UUID ────────────────────

/// Changing only the namespace must produce a different UUID. This proves
/// namespace is an independent axis of the v5 derivation.
#[test]
fn uuid_v5_different_namespace_produces_different_uuid() {
    // Two well-known RFC 4122 namespaces: DNS and URL.
    let src = r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [dns-id  (:wat::core::uuid::v5 "6ba7b810-9dad-11d1-80b4-00c04fd430c8" "example.com")
             url-id  (:wat::core::uuid::v5 "6ba7b811-9dad-11d1-80b4-00c04fd430c8" "example.com")]
            (:wat::core::if (:wat::core::= dns-id url-id) -> :wat::core::nil
              (:wat::kernel::println "SAME")
              (:wat::kernel::println "DIFFERENT"))))
    "#;
    let lines = run(src);
    assert_eq!(
        lines,
        vec!["\"DIFFERENT\""],
        "different namespace with same name must produce different UUIDs"
    );
}

// ─── C2: same namespace, different name → different UUID ────────────────────

/// Changing only the name must produce a different UUID. This proves
/// name is an independent axis of the v5 derivation.
#[test]
fn uuid_v5_different_name_produces_different_uuid() {
    let src = format!(
        r#"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a (:wat::core::uuid::v5 "{ns}" "name-alpha")
             b (:wat::core::uuid::v5 "{ns}" "name-beta")]
            (:wat::core::if (:wat::core::= a b) -> :wat::core::nil
              (:wat::kernel::println "SAME")
              (:wat::kernel::println "DIFFERENT"))))
    "#,
        ns = DNS_NAMESPACE
    );
    let lines = run(&src);
    assert_eq!(
        lines,
        vec!["\"DIFFERENT\""],
        "same namespace with different names must produce different UUIDs"
    );
}
