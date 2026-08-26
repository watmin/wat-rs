;; T22: top-level defn references a `def`-bound value.
;;
;; The RED gate for closure extraction's `def` arm. `closure_extract.rs`'s
;; Keyword walker resolves fn → unit-variant → type, and then RAISES
;; `Internal("captured `def`-bound name … not yet supported")` for anything
;; sitting in `runtime_def_values`. A top-level `def` read from a fn body is
;; exactly that shape — and it is the shape every `defservice` emits, because
;; each op's `:max-request-bytes` becomes a top-level `def`.
;;
;; The extraction must carry `:my::LIMIT`'s define into the prologue under its
;; ORIGINAL name: the body references it by Keyword, and Keyword references are
;; not rewritten by `rewrite_captures` (which only substitutes bare-Symbol
;; locals).
(:wat::core::def :my::LIMIT 512)

(:wat::core::defn :my::plus-limit [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ n :my::LIMIT))
