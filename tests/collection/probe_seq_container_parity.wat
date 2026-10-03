;; tests/collection/probe_seq_container_parity.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Each defn is named for its probe.
;; startup_beside type-checks all defns; tests eval individual functions.

(:wat::core::defn :p::first-pv [] -> wat.type/i64
  (:wat::core::first (wat.type/PersistentVector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::second-pv [] -> wat.type/i64
  (:wat::core::second (wat.type/PersistentVector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::third-pv [] -> wat.type/i64
  (:wat::core::third (wat.type/PersistentVector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :p::rest-pv [] -> wat.type/i64
  (:wat::core::length
    (:wat::core::rest (wat.type/PersistentVector :- [wat.type/i64] 10 20 30))))

(:wat::core::defn :p::conj-list [] -> wat.type/i64
  (:wat::core::length
    (:wat::core::conj (wat.type/List :- [wat.type/i64] 1 2) 3)))

(:wat::core::defn :p::first-watast [] -> wat.type/AST
  (:wat::core::first (:wat::core::quote (a b c))))

(:wat::core::defn :p::rest-watast [] -> wat.type/bool
  (:wat::core::let [_r (:wat::core::rest (:wat::core::quote (a b c)))] true))
