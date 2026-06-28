;; tests/collection/probe_hashmap_ctor_vector_symmetric_p8_bad.wat
;; Probe 8: zero type-args must fail arity check.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap)]
    0))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
