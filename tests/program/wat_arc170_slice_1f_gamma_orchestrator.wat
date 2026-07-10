;; tests/program/wat_arc170_slice_1f_gamma_orchestrator.wat — trivial canonical main.
;; Canonical [] -> :nil; body must DO something (arc-170 UselessMain wall).
;; Used for row_d (scope-drop cascade) via startup_beside(file!()).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::let [_argv (:wat::runtime::argv)] nil))
