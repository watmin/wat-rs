//! Error types for the name-resolution pass.
//!
//! [`UnresolvedReference`] — one failed call-head reference with source location.
//! [`ResolveError`] — the top-level error collecting all failures.

use crate::span::Span;
use std::fmt;

/// One unresolved reference, with context about where it appeared.
/// Stone 243.7e: each reference carries its source span so the collection
/// is location-complete without an outer span on [`ResolveError`].
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedReference {
    /// The keyword path that didn't resolve.
    pub path: String,
    /// Human-friendly context: a short phrase like "call head" or
    /// "macro call (not expanded)".
    pub context: &'static str,
    /// Source location of the offending keyword reference. `Span::unknown()`
    /// when the site genuinely has no recoverable location.
    pub span: Span,
}

/// Name-resolution errors.
#[derive(Debug)]
pub enum ResolveError {
    /// One or more references don't resolve. `unresolved` carries ALL
    /// failures so the user can fix them in a single pass.
    UnresolvedReferences(Vec<UnresolvedReference>),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::UnresolvedReferences(list) => {
                writeln!(f, "{} unresolved reference(s):", list.len())?;
                for r in list {
                    if r.span.is_unknown() {
                        writeln!(f, "  - {} ({})", r.path, r.context)?;
                    } else {
                        writeln!(f, "  - {} at {} ({})", r.path, r.span, r.context)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ResolveError {}
