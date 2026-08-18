;; Arc 278 #87 — NON-VACUITY: an acyclic rete-defn DAG must still load.
;; wrap → leaf, no back-edge. If this file is refused, the cycle walk is
;; treating any named call as recursion.

(:wat::rete::core::defn :probe::leaf [n <- :wat::core::i64] -> :wat::core::i64
  n)

(:wat::rete::core::defn :probe::wrap [n <- :wat::core::i64] -> :wat::core::i64
  (:probe::leaf n))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::wrap 1)))
