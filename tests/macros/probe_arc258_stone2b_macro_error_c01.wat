;; tests/macros/probe_arc258_stone2b_macro_error_c01.wat — NEGATIVE fixture for
;; probe_arc258_stone2b_macro_error.rs contract_01.
;; C01: a non-exhaustive cond with KEYWORD bodies is rejected.
(:wat::core::defn :user::f [] -> :wat::core::Keyword
  (:wat::core::cond
    ((:wat::core::= 1 1) :a)
    ((:wat::core::= 2 2) :b)))
