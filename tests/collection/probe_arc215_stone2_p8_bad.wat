;; tests/collection/probe_arc215_stone2_p8_bad.wat
;; Probe 8: mixed-type vector [1 "two"] must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length [1 "two"]))
