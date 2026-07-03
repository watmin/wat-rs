;; tests/collection/probe_arc216_stone4_predicate_composition_p5_bad.wat
;; Probe 5: to-holon on non-atomizable Fn type must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
    (:wat::holon::to-holon f)))
