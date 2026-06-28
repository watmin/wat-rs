;; tests/macros/probe_arc258_stone2b_macro_error_c02.wat — NEGATIVE fixture for
;; probe_arc258_stone2b_macro_error.rs contract_02.
;; C02: a non-exhaustive cond (string bodies) is rejected, naming :else.
(:wat::core::defn :user::g [] -> :wat::core::String
  (:wat::core::cond
    ((:wat::core::= 1 1) "x")
    ((:wat::core::= 2 2) "y")))
