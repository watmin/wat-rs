//! `Span` — re-exported from `wat-reader`. See `wat_reader::span` for docs.
//!
//! The `rust_caller_span!` macro is re-declared here so call sites in
//! the `wat` crate can continue using `crate::rust_caller_span!()` unchanged.

pub use wat_reader::span::*;

/// Expand to a [`Span`] naming the call-site's Rust source location.
/// Re-declared in this crate so `crate::rust_caller_span!()` resolves
/// inside `wat`'s own source. The implementation delegates to
/// `wat_reader::span::Span::new` via the re-exported `Span`.
///
/// (The canonical definition lives in `wat-reader/src/span.rs`.)
#[macro_export]
macro_rules! rust_caller_span {
    () => {
        $crate::span::Span::new(
            ::std::sync::Arc::new(format!("wat-rs/{}", file!())),
            line!() as i64,
            column!() as i64,
        )
    };
}
