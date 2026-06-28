;; tests/macros/probe_arc258_stone2_cond_macro.wat — co-located fixture for
;; probe_arc258_stone2_cond_macro.rs, slurped via startup_beside(file!()).
;;
;; Three named compute functions, one per contract test.
;; C01: first arm taken.
;; C02: else fallthrough.
;; C03: three-arm recursion, middle arm taken.

(:wat::core::defn :user::compute-1 [] -> :wat::core::i64
  (:wat::core::cond
    ((:wat::core::= 1 1) 10)
    (:else 20)))

(:wat::core::defn :user::compute-2 [] -> :wat::core::i64
  (:wat::core::cond
    ((:wat::core::= 1 2) 10)
    (:else 20)))

(:wat::core::defn :user::compute-3 [] -> :wat::core::i64
  (:wat::core::cond
    ((:wat::core::= 1 2) 10)
    ((:wat::core::= 2 2) 20)
    (:else 30)))

