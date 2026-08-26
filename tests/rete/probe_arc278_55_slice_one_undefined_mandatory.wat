;; tests/rete/probe_arc278_55_slice_one_undefined_mandatory.wat — EXPECTATIONS row 8: the
;; `:undefined` marker OMITTED. `:wat::rete::core::i64::+` is registered with a 4-param TypeScheme
;; (a, b, :undefined-marker, fallback); a 3-arg call must fail to type-check.
(:wat::core::defn :user::bad-fallback-call [] -> :wat::core::i64
  (:wat::rete::i64::+ 2 3 -1))
