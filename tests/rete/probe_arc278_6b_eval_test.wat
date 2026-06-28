;; tests/rete/probe_arc278_6b_eval_test.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines :test::big? for the eval-test probe.

(:wat::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::> n 100))

