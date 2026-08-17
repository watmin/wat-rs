//! Stone 118.3-B — a concrete container satisfies a PARAMETRIC surface (`Seqable<T>`).
//!
//! **The wat source is the co-located sibling fixture**
//! `probe_stone118_3b_seqable_parametric_satisfaction.wat`, driven via `call_beside_value` —
//! the repo's test-fixture scheme (never inlined as a Rust string).
//!
//! `src/check.rs`'s `(Parametric actual, Parametric expected)` arm (~14858) string-compared a
//! registered `extend-type` edge (stored VERBATIM using the SURFACE's own declared param name,
//! e.g. `:sq::Seqable<T>`) against the call site's rendered expected type (a fresh unification
//! var, e.g. `:sq::Seqable<?454>`) — `"<?454>" != "<T>"`, always, so NO concrete container could
//! ever satisfy a parametric surface bound. See
//! docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/{BRIEF,EXPECTATIONS,MEASURED}-118.3-B*.md.
//!
//! RED at HEAD (pre-fix): all four `param-*` fns below fail to type-check —
//! `:t118b::count-of: parameter #1 expects :t118b::Seqable<?N>; got :wat::core::<Container><…>`.
//! GREEN post-fix: all four dispatch (row 1); the two `bare-*` fns (arm 3, unrelated arm) are
//! unaffected either way — kept here as the row-2 companion so a future edit to arm 5 that
//! moves arm 3 is caught in the SAME file.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn expect_i64(fn_name: &str) -> i64 {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::i64(n) => n,
        other => panic!("{fn_name}: expected i64, got {other:?}"),
    }
}

/// Row 2 (★ STOP-3 guard) — the BARE (non-parametric) surface path, arm 3, byte-identical.
#[test]
fn bare_surface_still_dispatches_vector_and_persistent_vector() {
    assert_eq!(expect_i64(":t::bare-vector"), 3);
    assert_eq!(expect_i64(":t::bare-persistent-vector"), 4);
}

/// Row 1 — the PARAMETRIC surface `Seqable<T>` now dispatches for all four containers that
/// `extract_lazyable_elem` hardcodes: Vector, PersistentVector, List, Stream.
#[test]
fn parametric_surface_dispatches_vector() {
    assert_eq!(expect_i64(":t::param-vector"), 3);
}

#[test]
fn parametric_surface_dispatches_persistent_vector() {
    assert_eq!(expect_i64(":t::param-persistent-vector"), 4);
}

#[test]
fn parametric_surface_dispatches_list() {
    assert_eq!(expect_i64(":t::param-list"), 5);
}

#[test]
fn parametric_surface_dispatches_stream() {
    assert_eq!(expect_i64(":t::param-stream"), 2);
}
