;; tests/collection/probe_nth_persistent_vector.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()).
;;
;; Two defns: one on PersistentVector (the disconfirm), one on std Vector (regression guard).
(:wat::core::defn :test::pv-nth [] -> :wat::core::i64
  (:wat::core::nth
    (:wat::vector::conj (:wat::core::PersistentVector) 7)
    0))

(:wat::core::defn :test::vec-nth [] -> :wat::core::i64
  (:wat::core::nth (:wat::core::Vector :wat::core::i64 10 20 30) 1))
