;; tests/collection/probe_arc278_0c_persistent_parity.wat — co-located fixture for
;; the sibling probe (.rs), slurped via startup_beside(file!()). Named zero-arg
;; defns, one per assertion in `persistent_vector_transform_parity`.
;;
;; Arc 118.2a: `map`/`filter`/`take`/`drop` are LAZY — they return `(Stream :- [T])`
;; (never container-preserving); the fixtures below materialize back to a
;; PersistentVector via `into` before taking `length`, exactly mirroring the
;; original format!-driven exprs.

;; foldl / reduce-over-reverse (fn-first; return the accumulator)
;; Arc 118.B6b: `foldr` retired — `p2-foldr` renamed `p2-fold-reverse`, body now spelled
;; `(reduce f init (reverse coll))`, the composition that replaces it (`reverse`+`foldl`
;; wearing a name borrowed from Haskell, distinct only under laziness wat does not have).
(:wat::core::defn :t::p1-foldl [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :t::p2-fold-reverse [] -> :wat::core::i64
  (:wat::core::reduce
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x))
    0
    (:wat::core::reverse (:wat::core::PersistentVector 1 2 3))))

;; map / filter — materialize back to a PersistentVector via `into`.
(:wat::core::defn :t::p3-map [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::core::into (:wat::core::PersistentVector)
      (:wat::core::map
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))
        (:wat::core::PersistentVector 1 2 3)))))

(:wat::core::defn :t::p4-filter [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::core::into (:wat::core::PersistentVector)
      (:wat::core::filter
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::i64::> x 1))
        (:wat::core::PersistentVector 1 2 3)))))

;; reverse (type-preserving; head after reverse == 3 — get returns (Option :- [T]))
(:wat::core::defn :t::p5-reverse [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::vector::get (:wat::core::reverse (:wat::core::PersistentVector 1 2 3)) 0))

;; take / drop (coll-first; LAZY — materialize via `into`).
(:wat::core::defn :t::p6-take [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::core::into (:wat::core::PersistentVector) (:wat::core::take (:wat::core::PersistentVector 1 2 3) 2))))

(:wat::core::defn :t::p7-drop [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::core::into (:wat::core::PersistentVector) (:wat::core::drop (:wat::core::PersistentVector 1 2 3) 1))))

;; concat (two PersistentVectors → a PersistentVector)
(:wat::core::defn :t::p8-concat [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::core::concat (:wat::core::PersistentVector 1 2) (:wat::core::PersistentVector 3))))
