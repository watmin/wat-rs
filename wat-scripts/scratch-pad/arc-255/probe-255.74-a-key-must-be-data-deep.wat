;; probe-255.74 (deep) — the runtime guard `value_is_hashable` checks only the OUTER variant.
;; A vector is hashable; a vector holding a fn is not. Before 255.74: check passes, run panics.
(:wat::core::defn :probe::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [s (wat.type/HashSet :- [(wat.type/Vector :- [[wat.type/i64 :-> wat.type/i64]])]
         (wat.type/Vector :- [[wat.type/i64 :-> wat.type/i64]] :probe::inc))]
    (:wat::kernel::println "a vector of fns became a set element")))
