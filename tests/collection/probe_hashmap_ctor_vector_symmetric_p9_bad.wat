;; tests/collection/probe_hashmap_ctor_vector_symmetric_p9_bad.wat
;; Probe 9: missing V type-arg must fail arity check.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::HashMap :wat::core::keyword)]
    0))
