;; tests/macros/probe_arc258_stone2b_macro_error_c03.wat — NEGATIVE fixture for
;; probe_arc258_stone2b_macro_error.rs contract_03.
;; C03: macro-error surfaces its message in the diagnostic.
(:wat::core::defmacro :user::boom [] -> :wat::WatAST
  (:wat::core::macro-error "kaboom-sentinel-9173"))
(:wat::core::defn :user::h [] -> :wat::core::i64 (:user::boom))
