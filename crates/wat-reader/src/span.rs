//! `Span` — source-location metadata attached to every AST node.
//!
//! Arc 016 slice 1. Used by:
//! - The parser — captures the starting token's file/line/col onto
//!   each [`crate::ast::WatAST`] node it builds.
//! - The runtime — reads spans off call-form AST nodes and pushes
//!   them onto a thread-local call stack so `assertion-failed!` can
//!   populate `:wat::kernel::Failure.location` / `.frames`.
//! - The panic hook — reads the call stack to render Rust-style
//!   `file:line:col` output on test failure (arc 016 slice 3).
//!
//! # Equality and hashing
//!
//! `Span` equality is structural-transparent: two `Span` values ALWAYS
//! compare equal, and hashing is a no-op. This is intentional.
//!
//! Every `WatAST` variant carries a `Span`. The hash layer
//! ([`crate::hash::canonical_edn_wat`]) computes AST identity from
//! structural content — two ASTs with the same shape but different
//! source locations MUST hash to the same bytes. Same for derived
//! `PartialEq`: a parsed-at-runtime AST and a synthetic AST with the
//! same structure should compare equal regardless of where they came
//! from.
//!
//! The consequence: `Span::eq` returns `true` unconditionally;
//! `Span::hash` writes nothing. Downstream code that wants to reason
//! about source locations reads the Span's fields directly
//! (`file`, `line`, `col`); it never compares Span values for
//! equality.
//!
//! # File labels
//!
//! `file` is a best-effort label:
//! - Loaded files (stdlib baked, `load!`'d at startup, entry source
//!   via `wat <path>.wat`) use the path string.
//! - Test/eval parses that don't have a path use `<test>`, `<eval>`,
//!   `<repl>`, or a caller-supplied label.
//! - Synthetic forms (macro-expanded, runtime-constructed) use
//!   [`crate::rust_caller_span!()`] which carries the real Rust
//!   `file!():line!():col!()` of the constructing code.
//!
//! Stored as `Arc<String>` so spans clone cheaply — every AST node
//! share-clones the same string rather than allocating per-node.

use std::sync::Arc;

/// Source location attached to an AST node.
#[derive(Debug, Clone)]
pub struct Span {
    /// Best-effort file label. See module docs.
    pub file: Arc<String>,
    /// 1-indexed line number.
    pub line: i64,
    /// 1-indexed column number (char-count from line start).
    pub col: i64,
    /// 1-indexed end line (one past the last char of the token/form).
    /// Defaults to `line` when constructed via `Span::new` (degenerate end==start).
    pub end_line: i64,
    /// 1-indexed end column (one past the last char of the token/form).
    /// Defaults to `col` when constructed via `Span::new` (degenerate end==start).
    pub end_col: i64,
}

impl Span {
    /// Build a span with the given file label and 1-indexed position.
    /// `end_line` and `end_col` default to `line`/`col` (degenerate end==start).
    /// All existing call sites keep compiling unchanged.
    pub fn new(file: Arc<String>, line: i64, col: i64) -> Self {
        Span { file, line, col, end_line: line, end_col: col }
    }

    /// Build a span with explicit start AND end positions. Used by the lexer
    /// (to stamp each token's end) and the parser (to combine open..close for
    /// structural nodes). Arc 281.
    pub fn with_end(file: Arc<String>, line: i64, col: i64, end_line: i64, end_col: i64) -> Self {
        Span { file, line, col, end_line, end_col }
    }

}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col)
    }
}

/// Expand to a [`Span`] naming the call-site's Rust source
/// location. Used when the wat runtime invokes a user function
/// without a wat-source call site (test harness entry,
/// `compose_and_run` entry, internal iteration in `map`/`foldl`/
/// `fold`). Mirrors Rust's own backtrace convention — when a
/// Rust panic prints a stack backtrace, stdlib frames carry
/// `/rustc/.../library/core/.../function.rs:250:5` as their
/// source location. wat does the same: runtime-initiated calls
/// carry `wat-rs/src/<file>.rs:<line>:<col>` so a wat author
/// debugging the runtime knows exactly which Rust file invoked
/// their wat.
///
/// Arc 016 slice 3. Allocates a fresh `Arc<String>` per
/// invocation; the cost is only paid on failure-path rendering
/// (and fast, since it's in nanoseconds).
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

// Equality: always true. Span contributes nothing to structural
// equality; see module docs.
impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for Span {}

// Hash: no-op. Span contributes nothing to canonical hashes; see
// module docs.
impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

/// Arc 138 — render the file:line:col prefix for an error.
/// The prefix shape is `<file>:<line>:<col>: `.
///
/// Shared by `src/check.rs` (CheckError Display) and `src/types.rs`
/// (TypeError Display) — both were carrying identical private copies.
/// Arc 298.2: every span is now a real location; no sentinel elision.
pub fn span_prefix(span: &Span) -> String {
    format!("{}: ", span)
}

// ─── ToEdn ──────────────────────────────────────────────────────────────────
//
// The impl lives in `wat-reader` (where `Span` is defined) rather than in
// the `wat` crate because `ToEdn` is now defined in `wat-edn`. The orphan
// rule forbids implementing a foreign trait for a foreign type; keeping the
// impl in the crate that owns the type satisfies the rule.
//
// The produced form `{:file "…" :line N :col N}` is identical to
// `panic_hook::span_to_map` in the `wat` crate — the two must stay in sync.

impl wat_edn::ToEdn for Span {
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use std::borrow::Cow;
        use wat_edn::{Keyword, OwnedValue};
        OwnedValue::Map(vec![
            (
                OwnedValue::Keyword(Keyword::new("file")),
                OwnedValue::String(Cow::Owned(self.file.as_str().to_owned())),
            ),
            (
                OwnedValue::Keyword(Keyword::new("line")),
                OwnedValue::Integer(self.line),
            ),
            (
                OwnedValue::Keyword(Keyword::new("col")),
                OwnedValue::Integer(self.col),
            ),
        ])
    }
}
