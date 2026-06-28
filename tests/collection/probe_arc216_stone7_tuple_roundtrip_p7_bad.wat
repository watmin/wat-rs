;; tests/collection/probe_arc216_stone7_tuple_roundtrip_p7_bad.wat
;; Probe 7 negative: Tuple containing Fn rejects at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
     t (:wat::core::Tuple f "tag")]
    (:wat::holon::to-holon t)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
