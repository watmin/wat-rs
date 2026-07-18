;; tests/kernel/wat_harness_deps_user_main.wat — canonical nil :user::main shared by every
;; wat_harness_deps.rs test (they only care about dep-fn presence/behavior, not main's body).
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::let [_argv (:wat::runtime::argv)] nil))
