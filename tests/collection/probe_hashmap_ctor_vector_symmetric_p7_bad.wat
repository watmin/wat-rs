;; tests/collection/probe_hashmap_ctor_vector_symmetric_p7_bad.wat
;; Probe 7: odd pair count must fail type-check.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :foo)]
    0))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
