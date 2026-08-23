;; tests/program/wat_arc170_slice_1e_user_main_nil.wat — canonical :user::main fixture.
;; Canonical shape is [] -> :wat::core::nil; the body must DO something (arc-170 UselessMain wall).
;; Used for t1 (canonical freeze+invoke), t3 (argv ambient eval), t3 (current-thread eval).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "canonical main ran"))

;; just-eval probes (rubric) — t3_runtime_argv_ambient_eval_arm_produces_vector /
;; t3_runtime_current_thread_eval_arm_produces_string drive these via call_beside
;; instead of an inline ad-hoc expression.
(:wat::core::defn :probe::argv-compute [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::runtime::argv))

(:wat::core::defn :probe::current-thread-compute [] -> :wat::core::String
  (:wat::runtime::current-thread))
