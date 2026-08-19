//! Arc 118 — DISCONFIRMING PROBE for lazy seqs (Option C, the foundation strike).
//!
//! Decision (`118/DESIGN.md`): lazy seqs as **closures + memoized thunks** — `Value::wat__stream__Stream(Arc<Stream>)`,
//! `Stream = Empty | Cons{ head, LazyTail{ thunk: Arc<Function> } }`. Clojure-faithful (Surface C):
//! the family in `:wat::core::*`.
//!
//! ⚠ The decision above originally said "memoized thunks … force-once-cache (the
//! `ChildHandle.cached_exit` precedent)". Stone 118.B3 removed the cache: the thunks are deferred,
//! not memoized. Laziness is preserved; the RETENTION is not.
//!
//! RED at HEAD: `:wat::stream::cons` / `lazy-seq` / `seq-empty` do not exist (no lazy `Stream` value); `first`/`rest`
//! exist only for eager collections. GREEN when the six primitives land in `src/seq/` and `first`/`rest` extend to
//! `Value::Stream` — so a lazy cons-cell builds and traverses (the tail thunked, forced on `rest`).

use wat::freeze::startup_beside;

/// A lazy 2-element seq `(cons 1 (lazy-seq (cons 2 (lazy-seq (seq-empty)))))` builds and `first`/`rest`-traverses.
#[test]
fn lazy_seq_cons_first_rest_traverses() {
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "lazy-seq cons/first/rest should build a lazy seq and traverse it; got: {:?}",
        world.err()
    );
}
