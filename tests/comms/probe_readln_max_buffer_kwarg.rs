//! readln `:max-buffer-bytes` escape hatch — the exposed `readln` is a macro
//! over the kernel-restricted positional prime `readln'`.
//!
//! THE GAP: the value-framing accumulator caps at DEFAULT_MAX_FRAME_BYTES
//! (512 KiB). A caller with a legitimately larger message has no way to opt in.
//!
//! THE CONTRACT (kwargs is ALWAYS a macro):
//!  - `:wat::kernel::readln'` — kernel-restricted positional prime: optional
//!    leading max-bytes (`(readln' -> :T)` = default; `(readln' N -> :T)` = N).
//!  - `:wat::kernel::readln` — a defmacro wrapping readln': `(readln)` →
//!    `(readln' -> :T)`; `(readln :max-buffer-bytes N -> :T)` → `(readln' N -> :T)`.
//!    Forwards the `-> :T` annotation so the polymorphic return still infers.
//!
//! GREEN after: the macro expands it clean.
//! Both forms type-check in the co-located `.wat` fixture loaded via startup_beside.

use wat::freeze::startup_beside;

/// Type-check helper: loads the co-located .wat (which contains both readln forms).
/// Startup Ok iff BOTH forms type-check (they share one freeze pass).
fn startup_ok() -> bool {
    startup_beside(file!()).is_ok()
}

#[test]
fn readln_max_buffer_bytes_kwarg_type_checks() {
    // The escape hatch: opt into a 2 MiB frame cap. GREEN once readln is a macro over readln'.
    // The co-located .wat contains (readln :max-buffer-bytes N -> :T) in readln-with-max-buffer.
    assert!(
        startup_ok(),
        "(:wat::kernel::readln :max-buffer-bytes N) must type-check — \
         readln is the macro over the kernel prime readln'"
    );
}

#[test]
fn readln_plain_form_still_type_checks() {
    // Backward compat: the existing no-kwarg form must keep working.
    // The co-located .wat contains (readln -> :T) in readln-plain.
    assert!(
        startup_ok(),
        "(:wat::kernel::readln) must still type-check (default cap branch)"
    );
}
