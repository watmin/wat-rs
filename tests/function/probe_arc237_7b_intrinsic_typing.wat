;; tests/function/probe_arc237_7b_intrinsic_typing.wat
;; Arc 237 Stone 237.7b — ∀T intrinsic typing precision (empty?, contains?, get, conj).
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *.wat.bad files.

;; TIER A — empty? (∀T -> bool)
(:wat::core::defn :user::empty-q-vector [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::Vector :wat::core::i64)))

(:wat::core::defn :user::empty-q-hashset-false [] -> :wat::core::bool
  (:wat::core::empty? (:wat::core::HashSet :wat::core::i64 1 2)))

;; TIER A — contains? ((coll, elem) -> bool)
(:wat::core::defn :user::contains-q-vector-hit [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::Vector :wat::core::i64 1 2 3) 2))

;; TIER B — get ((coll, key) -> (Option :- [element]))
(:wat::core::defn :user::get-vector-precise [] -> :wat::core::i64
  (:wat::core::match (:wat::core::get (:wat::core::Vector :wat::core::i64 10 20 30) 1)
                     
                     ((:wat::core::Some x) (:wat::core::i64::+ x 5))
                     (:wat::core::None -1)))

;; TIER B — conj ((coll, elem) -> coll)
(:wat::core::defn :user::conj-vector-preserves [] -> :wat::core::i64
  (:wat::core::length (:wat::core::conj (:wat::core::Vector :wat::core::i64 1 2) 3)))
