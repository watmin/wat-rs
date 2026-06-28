;; tests/collection/probe_arc278_0d_transform_dispatch_parity.wat — co-located fixture,
;; slurped via startup_beside(file!()), asserting startup (type-check) succeeds.
;;
;; Contains the 8 transform-op defns over PersistentVector (foldl/foldr/map/filter/reverse/take/drop/concat)
;; and the 3 bare-typed-container defns — all must type-check clean (arc 278 stone 0d).

(:wat::core::defn :user::p-foldl [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :user::p-foldr [] -> :wat::core::i64
  (:wat::core::foldr
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :user::p-map [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::map
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
      (:wat::core::PersistentVector 1 2 3))))

(:wat::core::defn :user::p-filter [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::filter
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))
      (:wat::core::PersistentVector 1 2 3))))

(:wat::core::defn :user::p-rev [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::reverse (:wat::core::PersistentVector 1 2 3))))

(:wat::core::defn :user::p-take [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::take (:wat::core::PersistentVector 1 2 3) 2)))

(:wat::core::defn :user::p-drop [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::drop (:wat::core::PersistentVector 1 2 3) 1)))

(:wat::core::defn :user::p-concat [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::concat (:wat::core::PersistentVector 1 2 3) (:wat::core::PersistentVector 1 2 3))))

(:wat::core::defn :user::fold-bare-pv [xs <- :wat::core::PersistentVector] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    xs))

(:wat::core::defn :user::fold-bare-vec [xs <- :wat::core::Vector] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    xs))

(:wat::core::defn :user::map-bare-pv [xs <- :wat::core::PersistentVector] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::map
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
      xs)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
