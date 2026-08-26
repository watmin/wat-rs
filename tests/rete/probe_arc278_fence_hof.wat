;; tests/rete/probe_arc278_fence_hof.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). The 6a purity fence over HOF fold/map expressions:
;; foldl/map must classify pure∧deterministic when their fn-arg is pure, and the impurity of a
;; fn-arg must propagate (conditional purity, not a blanket HOF allow). Each entry quotes the
;; expr under test and hands it to the fence predicate — no eval of the quoted body happens.

(:wat::core::defn :user::pure-fold-is-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
                            (:wat::i64::+ acc (:wat::i64::* x x))) 0 xs))))

(:wat::core::defn :user::pure-fold-is-deterministic [] -> :wat::core::bool
  (:wat::rete::deterministic?
    (:wat::core::quote
      (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
                            (:wat::i64::+ acc (:wat::i64::* x x))) 0 xs))))

(:wat::core::defn :user::pure-map-is-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                          (:wat::i64::* x x)) xs))))

;; GUARD: an IMPURE fold (fn body calls println) must STILL be rejected — the fix must NOT
;; blanket-allow HOFs; the impurity of the fn-arg must propagate (conditional purity).
(:wat::core::defn :user::impure-fold-is-not-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
                            (:wat::core::do (:wat::kernel::println "side effect") acc)) 0 xs))))
