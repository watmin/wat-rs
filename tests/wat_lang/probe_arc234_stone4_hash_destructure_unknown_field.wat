;; tests/wat_lang/probe_arc234_stone4_hash_destructure_unknown_field.wat
;; Probe 5: unknown field on hash-destructure. Startup SUCCEEDS — the checker permits the form and
;; the runtime rejects it, so the UnknownField error arrives at EVAL (probe 5 invokes :user::compute).

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [{x :nonexistent} (:myapp::Voltage 5.0)]
      x))
