;; tests/collection/probe_seq_container_parity.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Each defn is named for its probe.
;; startup_beside type-checks all defns; tests eval individual functions.

(:wat::core::defn :p::first-pv [] -> :wat::core::i64
  (:wat::core::first (:wat::core::PersistentVector 10 20 30)))

(:wat::core::defn :p::second-pv [] -> :wat::core::i64
  (:wat::core::second (:wat::core::PersistentVector 10 20 30)))

(:wat::core::defn :p::third-pv [] -> :wat::core::i64
  (:wat::core::third (:wat::core::PersistentVector 10 20 30)))

(:wat::core::defn :p::rest-pv [] -> :wat::core::i64
  (:wat::core::PersistentVector/length
    (:wat::core::rest (:wat::core::PersistentVector 10 20 30))))

(:wat::core::defn :p::conj-list [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::conj (:wat::core::List 1 2) 3)))

(:wat::core::defn :p::first-watast [] -> :wat::WatAST
  (:wat::core::first (:wat::core::quote (a b c))))

(:wat::core::defn :p::rest-watast [] -> :wat::core::bool
  (:wat::core::let [_r (:wat::core::rest (:wat::core::quote (a b c)))] true))
