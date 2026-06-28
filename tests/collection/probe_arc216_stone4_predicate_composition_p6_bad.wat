;; tests/collection/probe_arc216_stone4_predicate_composition_p6_bad.wat
;; Probe 6: to-holon on nested Fn type must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [g (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::add n 1))]
    (:wat::holon::to-holon g)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
