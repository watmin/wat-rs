;; Stone 255.20 — POSITIVE: a scheme-less registry intrinsic called at its declared arity keeps
;; checking. `:wat::linkedlist::length` has a registry row (arity 1) and no TypeScheme.
(:wat::core::defn :probe::len [] -> :wat::core::i64
  (:wat::linkedlist::length (:wat::core::List 1 2 3)))
