//! Arc 118 — the lazy single-pass Stream foundation (`:wat::stream::*`).
//!
//! `Stream` is the runtime representation of a lazy, single-pass sequence (Option C:
//! closures + thunks). Carried as `Value::wat__stream__Stream(Arc<Stream>)`. A wat dialect
//! choice, NOT Clojure's persistent lazy-seq: consumed once, no rewind (see `LazyCell`).
//!
//! `LazyCell` is the deferred tail: a 0-arg wat closure, forced on demand. SINGLE-PASS —
//! you cannot rewind the *stream* (builder, 2026-06-27: *"you cannot walk back a stream"*).
//! The *cell*'s WHNF is cached: `empty?` / `first` / `rest` on the same thunk share one
//! force. That is not rewind — `mapv`/`into`/`stream->pvec` walk those three on every
//! cell, and without the cache `f` ran three times per element (the live MCP
//! increment-vs-item anomaly). Want the whole prefix again? Build the buffer yourself.
//!
//! `realize` drives a `Stream` to WHNF (Weak Head Normal Form): Empty or Cons. A Thunk
//! is forced by calling `apply_function` on the captured closure, then recursing until
//! the result is Empty|Cons. The SymbolTable is required because `apply_function`
//! needs it (the closure may call registered substrate functions).

use std::sync::{Arc, OnceLock};

use crate::span::Span;
use crate::value::Value;
use crate::value::{EvalBreak, Function, RuntimeError, RuntimeErrorKind};

/// A lazy sequence value — Empty terminator, strict-head Cons cell, or deferred Thunk.
///
/// Carried as `Value::wat__stream__Stream(Arc<Stream>)` in the wat runtime.
/// Clone is cheap (Arc bump on the outer Value; inner Stream is shared).
///
/// INVARIANT: `realize` always terminates with `Empty` or `Cons`; `Thunk` is the
/// only transitional state. A `Thunk` that returns a non-Stream `Value` is a runtime
/// error (the body of `lazy-seq` must evaluate to a Stream).
#[derive(Debug)]
pub enum Stream {
    /// Sequence terminator — the empty lazy seq. `first` → nil; `rest` → Empty; `empty?` → true.
    Empty,
    /// Strict head + tail (may itself be a Thunk — O(1) cons). `first` → head; `rest` → tail.
    Cons {
        head: Value,
        /// The tail is an `Arc<Stream>` (possibly a `Thunk`). `rest` returns it directly;
        /// the consumer forces on the NEXT `first`/`rest` call.
        tail: Arc<Stream>,
    },
    /// Deferred cell — the body of `(:wat::stream::lazy <body>)` has not been forced yet.
    /// `realize` forces once and caches WHNF on the cell (`empty?`/`first`/`rest` share it).
    Thunk(LazyCell),
    /// Arc 118.2a — a Rust-native deferred cell (see [`NativeLazyCell`]). Forced identically
    /// to `Thunk` (via `realize`'s loop) but the thunk is a plain Rust closure, not a wat
    /// `Function`/AST. Backs the lazy `map`/`filter`/`take`/`drop` intrinsics, which stay
    /// Rust-native for a bootstrap reason (see `NativeLazyCell`'s doc), not a wat closure.
    NativeThunk(NativeLazyCell),
}

/// A deferred cell — the unrealized tail of a single-pass stream.
///
/// `thunk` is a 0-arg wat closure `() -> Stream<T>` constructed by `(:wat::stream::lazy <body>)`.
///
/// SINGLE-PASS stream, WHNF-cached cell. You cannot rewind the stream (drop the
/// Cons and the head is gone). You CAN ask `empty?` then `first` then `rest` on
/// the same cell — `realize` caches Empty|Cons so that is one force, not three.
pub struct LazyCell {
    /// The captured 0-arg wat closure whose body is the `lazy-seq` body. Contains a
    /// `closed_env` carrying the environment at `lazy-seq` construction time.
    pub thunk: Arc<Function>,
    /// First `realize` wins. Shared across `LazyCell` clones (same cell).
    pub forced: Arc<OnceLock<Arc<Stream>>>,
}

impl LazyCell {
    pub fn new(thunk: Arc<Function>) -> Self {
        Self {
            thunk,
            forced: Arc::new(OnceLock::new()),
        }
    }
}

impl std::fmt::Debug for LazyCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyCell")
            .field("thunk", &"<closure>")
            .finish()
    }
}

/// Arc 118.2a — a Rust-native deferred cell for `map`/`filter`/`take`/`drop`.
///
/// **Why these stay Rust intrinsics instead of wat-over-primitives (the 118.2a DESIGN's
/// original preference, Decision B):** a sub-agent sweep of every `defmacro` body in the
/// stdlib found `:wat::core::take`/`drop` called INSIDE `:wat::core::defn`'s own macro body
/// (`wat/core.wat`, the `kwargs-lower`/argvec-splitting logic) and `:wat::core::map` called
/// inside `:wat::core::defrecord` / `:wat::holon::defrecord` / `:wat::service::defservice` /
/// `:wat::rete::defrule`'s macro bodies. Macro expansion (step 4 of the freeze pipeline) runs
/// BEFORE ordinary `defn`/`defclause` registration (step 6) makes a wat-defined function's REAL
/// body callable — a defclause only gets a nil-returning CHECKER stub at that point (Stone
/// 237.8b, `preregister_stdlib_defclause_stub`). Since `:wat::core::defn` itself depends on
/// `take`/`drop` to expand ANY `defn` — including a hypothetical wat-defined `take`/`drop`
/// itself — wat-defining them is circular and unbootstrappable, not just awkward. `map` faces
/// the softer (but still real) version: `defrecord`/`defservice` are invoked ~30+ times across
/// the stdlib (earliest: `Fault` at `core.wat`), so a wat-defined `map` would be an inert stub
/// at the exact moment those macros need real behavior.
///
/// The resolution: `map`/`take`/`drop` stay Rust intrinsics (bootstrap-safe, unconditionally
/// callable at every phase, exactly as before this arc) but their IMPLEMENTATION is changed to
/// build a lazy `Stream` via this native closure, instead of eagerly materializing a `Vec`. The
/// observable contract (lazy, `:wat::core::`-named, returns `Stream<T>`) is identical to a wat
/// implementation — only the mechanism differs. `filter` has no such macro-expansion-time
/// caller anywhere in the stdlib, so it ships as a genuine wat `defclause` (`wat/seq.wat`),
/// honoring Decision B wherever the bootstrap allows it.
///
/// Unlike [`LazyCell`] (a wat closure forced via `apply_function`), this thunk is a plain Rust
/// closure — no wat AST or `closed_env` involved. It still receives the `SymbolTable` + the
/// call-site `Span` as PARAMETERS (not captured) purely so it can invoke `apply_function` on a
/// captured user `Function` (the `f`/`pred` argument to map/filter) — the SymbolTable itself
/// is never stashed inside the closure.
/// Deferred body of a [`NativeLazyCell`]: SymbolTable + span in, WHNF Stream out.
type NativeThunk = Arc<
    dyn Fn(&crate::value::SymbolTable, &Span) -> Result<Arc<Stream>, EvalBreak> + Send + Sync,
>;

#[derive(Clone)]
pub struct NativeLazyCell {
    pub thunk: NativeThunk,
    pub forced: Arc<OnceLock<Arc<Stream>>>,
}

impl NativeLazyCell {
    pub fn new(thunk: NativeThunk) -> Self {
        Self {
            thunk,
            forced: Arc::new(OnceLock::new()),
        }
    }
}

impl std::fmt::Debug for NativeLazyCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeLazyCell")
            .field("thunk", &"<native closure>")
            .finish()
    }
}

/// Force a `Stream` to Weak Head Normal Form (WHNF): `Empty` or `Cons`.
///
/// - `Empty` / `Cons` → returned as-is (already in WHNF).
/// - `Thunk` / `NativeThunk` → force the closure, cache WHNF on the cell, recurse
///   if the result is still a thunk. `empty?`/`first`/`rest` on the same cell
///   share that cache. Dropping the Cons and trying to recover the head is still
///   impossible — that is the single-pass rule.
///
/// Error handling: if the thunk body returns a non-Stream value or errors through
/// `apply_function`, the error propagates as an `EvalBreak`.
///
/// `sym`: the symbol table, required by `apply_function` for registered-name dispatch.
/// `span`: the call-site span, threaded into `apply_function` for error location.
pub fn realize(
    seq: &Arc<Stream>,
    sym: &crate::value::SymbolTable,
    span: &Span,
) -> Result<Arc<Stream>, EvalBreak> {
    // Iterative loop to handle consecutive Thunks without blowing the stack.
    let mut current = Arc::clone(seq);
    loop {
        match current.as_ref() {
            Stream::Empty | Stream::Cons { .. } => return Ok(current),
            Stream::Thunk(cell) => {
                if let Some(s) = cell.forced.get() {
                    current = Arc::clone(s);
                    continue;
                }
                let result = crate::runtime::apply_function(
                    Arc::clone(&cell.thunk),
                    vec![],
                    sym,
                    span.clone(),
                )
                .map_err(EvalBreak::from)?;
                let next = match result {
                    Value::wat__stream__Stream(s) => s,
                    other => {
                        return Err(EvalBreak::from(RuntimeError::new(
                            span.clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: ":wat::stream::lazy (thunk force)".into(),
                                expected: "wat::stream::Stream",
                                got: Box::new(crate::value::ValueSnapshot::of(&other)),
                            },
                        )))
                    }
                };
                current = Arc::clone(cell.forced.get_or_init(|| next));
            }
            Stream::NativeThunk(cell) => {
                if let Some(s) = cell.forced.get() {
                    current = Arc::clone(s);
                    continue;
                }
                let next = (cell.thunk)(sym, span)?;
                current = Arc::clone(cell.forced.get_or_init(|| next));
            }
        }
    }
}

/// Arc 118.2a — convert an already-resident, eager sequence container (`Vector<T>` /
/// `List<T>` / `PersistentVector<T>`) into a fully-realized `Stream::Cons` chain (every cell
/// already `Cons`, no thunks). This is a pure reshape, never a deferred call: the container's
/// elements are already in memory, so building the chain touches no user code and violates no
/// laziness guarantee. Iterative (builds tail-to-head) to avoid recursion-depth limits on large
/// containers. Returns `None` if `v` is not one of the three eager containers — callers that
/// also want to accept an existing `Stream` should try [`value_as_stream`] instead.
pub(crate) fn eager_container_to_stream(v: &Value) -> Option<Arc<Stream>> {
    fn chain_from_slice<'a>(
        xs: impl DoubleEndedIterator<Item = &'a Value> + ExactSizeIterator,
    ) -> Arc<Stream> {
        let mut tail = Arc::new(Stream::Empty);
        for x in xs.rev() {
            tail = Arc::new(Stream::Cons {
                head: x.clone(),
                tail,
            });
        }
        tail
    }
    match v {
        Value::Vec(xs) => Some(chain_from_slice(xs.iter())),
        Value::wat__core__List(xs) => {
            let snapshot: Vec<Value> = xs.iter().cloned().collect();
            Some(chain_from_slice(snapshot.iter()))
        }
        Value::wat__core__PersistentVector(pv) => {
            let snapshot: Vec<Value> = pv.iter().cloned().collect();
            Some(chain_from_slice(snapshot.iter()))
        }
        _ => None,
    }
}

/// Arc 118.2a — the shared "any seqable" normalizer for the lazy `map`/`filter`/`take`/`drop`
/// intrinsics: a `Value::wat__stream__Stream` is returned as-is (already lazy, `Arc` bump
/// only); `Vector`/`List`/`PersistentVector` are converted via [`eager_container_to_stream`].
/// `None` for anything else — the caller raises `TypeMismatch`.
pub(crate) fn value_as_stream(v: &Value) -> Option<Arc<Stream>> {
    match v {
        Value::wat__stream__Stream(s) => Some(Arc::clone(s)),
        other => eager_container_to_stream(other),
    }
}
