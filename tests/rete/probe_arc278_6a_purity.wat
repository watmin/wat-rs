;; tests/rete/probe_arc278_6a_purity.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines test functions for purity/determinism classification.

(:wat::core::defn :test::pure-double [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::* n 2))

(:wat::core::defn :test::nondet-uuid [] -> :wat::core::Uuid
  (:wat::core::Uuid/v4))

(:wat::core::defn :test::io-fn [] -> :wat::io::IOReader
  (:wat::io::IOReader/open-file "x"))

(:wat::core::defn :test::countdown [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::<= n 0)
    0
    (:test::countdown (:wat::core::- n 1))))

