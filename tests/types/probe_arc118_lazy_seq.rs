//! Arc 118 — DISCONFIRMING PROBE for lazy seqs (Option C, the foundation strike).
//!
//! Decision (`118/DESIGN.md`): lazy seqs as **closures + memoized thunks** — `Value::wat__stream__Stream(Arc<Stream>)`,
//! `Stream = Empty | Cons{ head, LazyTail{ thunk: Arc<Function>, forced: OnceLock } }`, force-once-cache (the
//! `ChildHandle.cached_exit` precedent). Clojure-faithful (Surface C): the family in `:wat::core::*`.
//!
//! RED at HEAD: `:wat::stream::cons` / `lazy-seq` / `seq-empty` do not exist (no lazy `Stream` value); `first`/`rest`
//! exist only for eager collections. GREEN when the six primitives land in `src/seq/` and `first`/`rest` extend to
//! `Value::Stream` — so a lazy cons-cell builds and traverses (the tail thunked, forced on `rest`).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// A lazy 2-element seq `(cons 1 (lazy-seq (cons 2 (lazy-seq (seq-empty)))))` builds and `first`/`rest`-traverses.
#[test]
fn lazy_seq_cons_first_rest_traverses() {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let [s (:wat::stream::cons 1
                                 (:wat::stream::lazy
                                   (:wat::stream::cons 2
                                     (:wat::stream::lazy (:wat::stream::empty)))))]
            (:wat::core::do
              (:wat::kernel::pprintln (:wat::core::first s))
              (:wat::kernel::pprintln (:wat::core::first (:wat::core::rest s)))
              nil)))
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        world.is_ok(),
        "lazy-seq cons/first/rest should build a lazy seq and traverse it; got: {:?}",
        world.err()
    );
}
