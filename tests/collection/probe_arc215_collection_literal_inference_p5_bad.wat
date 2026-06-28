;; tests/collection/probe_arc215_collection_literal_inference_p5_bad.wat
;; Probe 5: mixed-value-type map {a 1 :b "two"} must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length {:a 1 :b "two"}))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
