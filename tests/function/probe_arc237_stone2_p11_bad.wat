;; tests/function/probe_arc237_stone2_p11_bad.wat — NEGATIVE probe 11: empty defclause.
;; defclause with 0 clauses must be rejected at parse/registration. startup MUST fail.

(:wat::core::defclause :my::empty)
