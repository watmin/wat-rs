;; Non-canonical :user::main return type (i64 instead of nil) — fires
;; BareLegacyMainSignature at type-check time. See
;; tests/cli/wat_cli.rs::wrong_arg_type_user_main_rejected.
(:wat::core::defn :user::main [] -> :wat::core::i64
  ())
