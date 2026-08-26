;; tests/wat_lang/wat_core_cond.wat — co-located fixture for the sibling probe (.rs).
;; Covers positive tests for :wat::core::cond.
;; Negative tests use separate *.wat.bad files.

; test: cond_first_arm_matches
(:wat::core::defn :t::cond-first [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 1) "first")
    ((:wat::core::= 2 2) "second")
    (:else "none")))

; test: cond_middle_arm_matches
(:wat::core::defn :t::cond-middle [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 2) "first")
    ((:wat::core::= 3 3) "middle")
    ((:wat::core::= 4 5) "third")
    (:else "none")))

; test: cond_falls_through_to_else
(:wat::core::defn :t::cond-else [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 2) "first")
    ((:wat::core::= 3 4) "second")
    (:else "defaulted")))

; test: cond_with_single_else_only
(:wat::core::defn :t::cond-only-else [] -> :wat::core::i64
  (:wat::core::cond -> :wat::core::i64
    (:else 42)))

; test: cond_dispatches_on_bound_value
(:wat::core::defn :t::label [code <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= code 1) "[runtime error]")
    ((:wat::core::= code 2) "[panic]")
    ((:wat::core::= code 3) "[startup error]")
    (:else "[nonzero exit]")))
(:wat::core::defn :t::cond-dispatch [] -> :wat::core::String (:t::label 3))

; test: cond_preserves_tail_call
(:wat::core::defn :t::countdown [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::cond -> :wat::core::i64
    ((:wat::core::= n 0) 0)
    ((:wat::core::< n 0) -1)
    (:else (:t::countdown (:wat::i64::- n 1)))))
(:wat::core::defn :t::cond-tail [] -> :wat::core::i64 (:t::countdown 100000))

; test: cond_composes_with_other_cond
(:wat::core::defn :t::cond-nested [] -> :wat::core::String
  (:wat::core::cond -> :wat::core::String
    ((:wat::core::= 1 2) "outer-first")
    ((:wat::core::= 1 1)
      (:wat::core::cond -> :wat::core::String
        ((:wat::core::= 7 8) "inner-first")
        (:else "inner-else")))
    (:else "outer-else")))
