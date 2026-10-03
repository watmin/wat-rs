;; Stone 255.20 — POSITIVE: a scheme-less registry intrinsic called at its declared arity keeps
;; checking. `:wat::linkedlist::length` has a registry row (arity 1) and no TypeScheme.
(:wat::core::defn :probe::len [] -> wat.type/i64
  (:wat::core::length (wat.type/List :- [wat.type/i64] 1 2 3)))
