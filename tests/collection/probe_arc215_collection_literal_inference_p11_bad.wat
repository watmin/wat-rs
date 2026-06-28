;; tests/collection/probe_arc215_collection_literal_inference_p11_bad.wat
;; Probe 11: mixed-element-type set #{1 :foo "x"} must fail at check with TypeMismatch.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length #{1 :foo "x"}))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
