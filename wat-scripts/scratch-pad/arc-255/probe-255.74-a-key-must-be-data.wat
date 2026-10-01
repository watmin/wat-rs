;; probe-255.74 — a set element / map key must be key-eligible (is_atomizable). Before 255.74
;; the checker admits a fn as a HashSet element and the run panics in `impl Hash for Value`.
(:wat::core::defn :probe::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [s (wat.type/HashSet :- [[wat.type/i64 :-> wat.type/i64]] :probe::inc)]
    (:wat::kernel::println "a fn became a set element")))
