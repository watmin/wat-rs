//! Arc 118 — lazy-seq foundation.
//!
//! `Seq` is the runtime representation of a lazy sequence (Clojure-faithful Option C:
//! closures + memoized thunks). Carried as `Value::wat__core__Seq(Arc<Seq>)`.
//!
//! `LazyCell` is the deferred tail: a 0-arg wat closure, forced on demand. SINGLE-PASS —
//! **no memoization** (builder, 2026-06-27: *"you cannot walk back a stream … core does not
//! ship it"*). A stream is consumed once; re-forcing is a consumer error. Want rewind? Build
//! the buffer yourself — core ships the honest, O(1)-memory primitive.
//!
//! `realize` drives a `Seq` to WHNF (Weak Head Normal Form): Empty or Cons. A Thunk
//! is forced by calling `apply_function` on the captured closure, then recursing until
//! the result is Empty|Cons. The SymbolTable is required because `apply_function`
//! needs it (the closure may call registered substrate functions).

use std::sync::Arc;

use crate::span::Span;
use crate::value::{Function, RuntimeError, RuntimeErrorKind, EvalBreak};
use crate::value::Value;

/// A lazy sequence value — Empty terminator, strict-head Cons cell, or deferred Thunk.
///
/// Carried as `Value::wat__core__Seq(Arc<Seq>)` in the wat runtime.
/// Clone is cheap (Arc bump on the outer Value; inner Seq is shared).
///
/// INVARIANT: `realize` always terminates with `Empty` or `Cons`; `Thunk` is the
/// only transitional state. A `Thunk` that returns a non-Seq `Value` is a runtime
/// error (the body of `lazy-seq` must evaluate to a Seq).
#[derive(Debug)]
pub enum Seq {
    /// Sequence terminator — the empty lazy seq. `first` → nil; `rest` → Empty; `empty?` → true.
    Empty,
    /// Strict head + tail (may itself be a Thunk — O(1) cons). `first` → head; `rest` → tail.
    Cons {
        head: Value,
        /// The tail is an `Arc<Seq>` (possibly a `Thunk`). `rest` returns it directly;
        /// the consumer forces on the NEXT `first`/`rest` call.
        tail: Arc<Seq>,
    },
    /// Deferred cell — the body of `(:wat::core::lazy-seq <body>)` has not been forced yet.
    /// `realize` calls the thunk closure and returns the result; nothing is cached (single-pass).
    Thunk(LazyCell),
}

/// A deferred cell — the unrealized tail of a single-pass stream.
///
/// `thunk` is a 0-arg wat closure `() -> Seq<T>` constructed by `(:wat::core::lazy-seq <body>)`.
///
/// SINGLE-PASS — **no memoization** (builder, 2026-06-27: *"you cannot walk back a stream — if
/// you want this you gotta write it … core does not ship it"*). `realize` forces the thunk and
/// returns; nothing is cached. Re-forcing is a consumer error (walk linearly). Holding the head
/// pins NOTHING, so streaming is unconditionally O(1) memory — terabytes through a few GB.
pub struct LazyCell {
    /// The captured 0-arg wat closure whose body is the `lazy-seq` body. Contains a
    /// `closed_env` carrying the environment at `lazy-seq` construction time.
    pub thunk: Arc<Function>,
}

impl std::fmt::Debug for LazyCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyCell").field("thunk", &"<closure>").finish()
    }
}

/// Force a `Seq` to Weak Head Normal Form (WHNF): `Empty` or `Cons`.
///
/// - `Empty` / `Cons` → returned as-is (already in WHNF).
/// - `Thunk` → call the 0-arg closure via `apply_function`, expect `Value::wat__core__Seq`
///   result, recurse (the forced result may itself be a Thunk). NOT cached.
///
/// SINGLE-PASS — no memoization (builder, 2026-06-27). The thunk runs each time it is
/// reached; a stream is walked once and can't be rewound. Re-forcing the same cell is a
/// consumer error, not a supported operation — want rewind, build the buffer yourself.
///
/// Error handling: if the thunk body returns a non-Seq value or errors through
/// `apply_function`, the error propagates as an `EvalBreak`.
///
/// `sym`: the symbol table, required by `apply_function` for registered-name dispatch.
/// `span`: the call-site span, threaded into `apply_function` for error location.
pub fn realize(
    seq: &Arc<Seq>,
    sym: &crate::value::SymbolTable,
    span: &Span,
) -> Result<Arc<Seq>, EvalBreak> {
    // Iterative loop to handle consecutive Thunks without blowing the stack.
    let mut current = Arc::clone(seq);
    loop {
        match current.as_ref() {
            Seq::Empty | Seq::Cons { .. } => return Ok(current),
            Seq::Thunk(cell) => {
                // SINGLE-PASS: force the thunk, NO caching. A stream can't be walked back;
                // re-forcing is a consumer error (walk linearly). The result may itself be a
                // Thunk — the loop drives it to WHNF.
                let result = crate::runtime::apply_function(
                    Arc::clone(&cell.thunk),
                    vec![],
                    sym,
                    span.clone(),
                ).map_err(EvalBreak::from)?;
                current = match result {
                    Value::wat__core__Seq(s) => s,
                    other => return Err(EvalBreak::from(RuntimeError {
                        span: span.clone(),
                        kind: RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::lazy-seq (thunk force)".into(),
                            expected: "wat::core::Seq",
                            got: Box::new(crate::value::ValueSnapshot::of(&other)),
                        },
                    })),
                };
            }
        }
    }
}
