;; tests/function/probe_arc247_hof_fn_first.wat
;; Arc 247 — Clojure-honest seq-HOF order (fn-first).
;; Co-located fixture, slurped via startup_beside(file!()).
;; coll-first negative case is in probe_arc247_hof_coll_first.wat.bad.

;; REGRESSION — variadic plus uses foldl internally; flip must not change result
(:wat::core::defn :user::regression-plus [] -> :wat::core::bool
  (:wat::core::= (:wat::core::+ 1 2 3 4) 10))

;; MINT-CONFIRMERS — fn-first order (map f xs).
;; Arc 118.2a — `map` flipped LAZY (returns Stream); this probe's intent (fn-first ARGUMENT
;; ORDER, unchanged by the flip) still holds — force via `mapv` before the equality check
;; so the comparison is against a concrete Vector, same as before.
(:wat::core::defn :user::mint-map-fn-first [] -> :wat::core::bool
  (:wat::core::= (:wat::core::mapv
                   (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                     (:wat::i64::+ x 1))
                   [1 2 3])
                 [2 3 4]))

;; MINT-CONFIRMERS — fn-first order (filter pred xs). Arc 118.2a — same note as map above.
(:wat::core::defn :user::mint-filter-fn-first [] -> :wat::core::bool
  (:wat::core::= (:wat::core::filterv
                   (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
                     (:wat::i64::> x 1))
                   [1 2 3])
                 [2 3]))

;; MINT-CONFIRMERS — fn-first order (foldl f init xs)
(:wat::core::defn :user::mint-foldl-fn-first [] -> :wat::core::bool
  (:wat::core::= (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                     (:wat::i64::+ acc x))
                   0
                   [1 2 3])
                 6))
