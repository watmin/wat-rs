;; tests/collection/probe_arc216_stone1_hashset_roundtrip_p9_bad.wat
;; Probe 9: to-holon on Fn type must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
    (:wat::holon::to-holon f)))
