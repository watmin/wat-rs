;; A well-formed program `wat --check` accepts. See tests/cli/wat_cli.rs
;; (arc 115 slice 1 `wat --check` mode):
;; check_mode_exits_zero_on_good_program,
;; check_output_without_check_flag_is_usage_error.
(:wat::core::defn :user::main [] -> :wat::core::nil
  ())
