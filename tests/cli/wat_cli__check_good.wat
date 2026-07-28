;; A well-formed program `wat --check` accepts. See tests/cli/wat_cli.rs
;; (arc 115 slice 1 `wat --check` mode):
;; check_mode_exits_zero_on_good_program,
;; check_output_without_check_flag_is_usage_error.
;;
;; Arc 179: `()` retired as a value; the body was originally the bare `()`
;; no-op. A bare `nil` body trips the pre-existing UselessMain wall
;; (src/freeze.rs:1433 `validate_user_main_not_useless` — ":user::main body
;; is the bare `nil` literal"), which `()` had been silently dodging simply
;; by not being a `WatAST::NilLit` node. A real (harmless, --check never
;; runs it) body keeps this a well-formed program instead.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "check-good"))
