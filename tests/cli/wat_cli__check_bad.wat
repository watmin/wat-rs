;; A program `wat --check` rejects: produces 2 type-check errors — one
;; CommCallOutOfPosition (send to an undeclared comm) + one
;; ReturnTypeMismatch. See tests/cli/wat_cli.rs (arc 115 slice 1
;; `wat --check` mode): check_mode_exits_nonzero_on_bad_program,
;; check_output_edn_emits_record_per_diagnostic,
;; check_output_json_emits_record_per_diagnostic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::send no-such-thing 42))
