//! S-expression parser — re-exported from `wat-reader`. See `wat_reader::parser` for docs.
//!
//! The `parse_one!` and `parse_all!` macros are re-declared here so call sites
//! in the `wat` crate can continue using `crate::parse_one!()` unchanged.

pub use wat_reader::parser::*;

/// Parse one form, auto-capturing the call-site Rust source location.
/// Re-declared in this crate so `crate::parse_one!()` resolves inside
/// `wat`'s own source. Delegates to `parse_one_with_file` (re-exported
/// from `wat-reader`).
#[macro_export]
macro_rules! parse_one {
    ($src:expr $(,)?) => {
        $crate::parser::parse_one_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

/// Parse all forms, auto-capturing the call-site Rust source location.
/// Re-declared in this crate so `crate::parse_all!()` resolves inside
/// `wat`'s own source. Delegates to `parse_all_with_file` (re-exported
/// from `wat-reader`).
#[macro_export]
macro_rules! parse_all {
    ($src:expr $(,)?) => {
        $crate::parser::parse_all_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}
