;; tests/function/probe_diagnostic_non_vector.wat — NEGATIVE probe 8: non-vector spread arg.
;; apply with i64 (42) as the trailing spread arg must reject. Startup SUCCEEDS — the spread-arg
;; check is dynamic, so the error arrives at EVAL (probe 8 starts the world up, then invokes).

(:wat::core::defn :user::bad [] -> :wat::core::i64
  (:wat::core::apply 
    (:wat::core::keyword/from-string "wat::core::i64::+")
    42))
