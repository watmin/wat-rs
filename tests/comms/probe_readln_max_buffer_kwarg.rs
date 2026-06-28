//! readln `:max-buffer-bytes` escape hatch — the exposed `readln` is a macro
//! over the kernel-restricted positional prime `readln'`.
//!
//! THE GAP: the value-framing accumulator caps at DEFAULT_MAX_FRAME_BYTES
//! (512 KiB). A caller with a legitimately larger message has no way to opt in.
//!
//! THE CONTRACT (kwargs is ALWAYS a macro):
//!  - `:wat::kernel::readln'` — kernel-restricted positional prime: optional
//!    leading max-bytes (`(readln' -> :T)` = default; `(readln' N -> :T)` = N).
//!  - `:wat::kernel::readln` — a defmacro wrapping readln': `(readln -> :T)` →
//!    `(readln' -> :T)`; `(readln :max-buffer-bytes N -> :T)` → `(readln' N -> :T)`.
//!    Forwards the `-> :T` annotation so the polymorphic return still infers.
//!
//! RED at HEAD: `readln` is a positional intrinsic expecting exactly `[-> :T]`;
//! `(readln :max-buffer-bytes N -> :T)` is a wrong-shape call → check error, so
//! `startup_from_source` returns Err. GREEN after: the macro expands it clean.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// A body that uses the kwarg form of readln. It need only TYPE-CHECK (the
/// StdInService isn't running here) — startup does the check pass.
fn startup_ok(body: &str) -> bool {
    let src = format!(
        "(:wat::core::defn :user::main [] -> :wat::core::nil {body})"
    );
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new())).is_ok()
}

#[test]
fn readln_max_buffer_bytes_kwarg_type_checks() {
    // The escape hatch: opt into a 2 MiB frame cap. RED at HEAD (readln is a
    // positional intrinsic; the kwarg shape fails to check). GREEN once readln
    // is a macro over readln'.
    let body = "(:wat::core::let \
                  [_line (:wat::kernel::readln :max-buffer-bytes (:wat::core::i64::* 2 (:wat::core::i64::* 1024 1024)) -> :wat::core::String)] \
                  nil)";
    assert!(
        startup_ok(body),
        "(:wat::kernel::readln :max-buffer-bytes N -> :T) must type-check — \
         readln is the macro over the kernel prime readln'"
    );
}

#[test]
fn readln_plain_form_still_type_checks() {
    // Backward compatibility: the existing no-kwarg form must keep working
    // (the macro's default branch → (readln' -> :T)).
    let body = "(:wat::core::let \
                  [_line (:wat::kernel::readln -> :wat::core::String)] \
                  nil)";
    assert!(
        startup_ok(body),
        "(:wat::kernel::readln -> :T) must still type-check (default cap branch)"
    );
}
