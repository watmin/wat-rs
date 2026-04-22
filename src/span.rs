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
//!   [`Span::unknown`] which labels as `<synthetic>`.
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
}

impl Span {
    /// Sentinel span for internally-constructed forms that have no
    /// source location (test helpers, runtime-built AST nodes). The
    /// file label `<runtime>` surfaces in backtraces only when an
    /// internal AST node reaches a call site — rare in practice,
    /// since real runtime-initiated invocations use
    /// [`crate::rust_caller_span`] which carries the Rust
    /// `file!()`:`line!()`:`column!()` instead.
    pub fn unknown() -> Self {
        Span {
            file: Arc::new("<runtime>".to_string()),
            line: 0,
            col: 0,
        }
    }

    /// Build a span with the given file label and 1-indexed position.
    pub fn new(file: Arc<String>, line: i64, col: i64) -> Self {
        Span { file, line, col }
    }

    /// `true` iff this is the synthetic sentinel — useful for error
    /// messages that want to skip "at <synthetic>:0:0" noise.
    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.col == 0
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
