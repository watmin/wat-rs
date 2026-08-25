;; tests/wat_lang/wat_arc098_form_matches_runtime.wat — co-located fixture.
;; Arc 098 slice 2 — :wat::form::matches? runtime walker end-to-end tests.
;; All tests evaluate named :t::testN-* fns returning bool via eval_in_frozen.

(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defstruct :test::Other
  [x <- :wat::core::i64])

; worked_example_matches — Grace 7.5 matches
(:wat::core::defn :t::test1-worked [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)]
    (:wat::form::matches? p
      (:test::PaperResolved
        (= ?outcome :outcome)
        (= ?grace-residue :grace-residue)
        (= ?outcome "Grace")
        (> ?grace-residue 5.0)))))

; worked_example_rejects_low_residue — Grace 3.0 does not match
(:wat::core::defn :t::test2-low-residue [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p
      (:test::PaperResolved
        (= ?outcome :outcome)
        (= ?grace-residue :grace-residue)
        (= ?outcome "Grace")
        (> ?grace-residue 5.0)))))

; worked_example_rejects_wrong_outcome — Loss does not match Grace
(:wat::core::defn :t::test3-wrong-outcome [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Loss" :grace-residue 7.5)]
    (:wat::form::matches? p
      (:test::PaperResolved
        (= ?outcome :outcome)
        (= ?grace-residue :grace-residue)
        (= ?outcome "Grace")
        (> ?grace-residue 5.0)))))

; comparison_lt — 7.5 < 5.0 = false; 3.0 < 5.0 = true
(:wat::core::defn :t::test4-lt-high [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (< ?gr 5.0)))))
(:wat::core::defn :t::test4-lt-low [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (< ?gr 5.0)))))

; comparison_gt — 7.5 > 5.0 = true; 3.0 > 5.0 = false
(:wat::core::defn :t::test4-gt-high [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (> ?gr 5.0)))))
(:wat::core::defn :t::test4-gt-low [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (> ?gr 5.0)))))

; comparison_le — 7.5 <= 5.0 = false; 3.0 <= 5.0 = true
(:wat::core::defn :t::test4-le-high [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (<= ?gr 5.0)))))
(:wat::core::defn :t::test4-le-low [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (<= ?gr 5.0)))))

; comparison_ge — 7.5 >= 5.0 = true; 3.0 >= 5.0 = false
(:wat::core::defn :t::test4-ge-high [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (>= ?gr 5.0)))))
(:wat::core::defn :t::test4-ge-low [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved (= ?gr :grace-residue) (>= ?gr 5.0)))))

; not_eq_works — Loss != Grace
(:wat::core::defn :t::test5-not-eq [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Loss" :grace-residue 1.0)]
    (:wat::form::matches? p
      (:test::PaperResolved
        (= ?o :outcome)
        (:not= ?o "Grace")))))

; and_both_must_hold
(:wat::core::defn :t::test6-and-pass [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome) (= ?gr :grace-residue)
      (:and (= ?o "Grace") (> ?gr 5.0))))))
(:wat::core::defn :t::test6-and-fail-residue [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome) (= ?gr :grace-residue)
      (:and (= ?o "Grace") (> ?gr 5.0))))))
(:wat::core::defn :t::test6-and-fail-outcome [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Loss" :grace-residue 7.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome) (= ?gr :grace-residue)
      (:and (= ?o "Grace") (> ?gr 5.0))))))

; or_at_least_one_must_hold
(:wat::core::defn :t::test7-or-low [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?gr :grace-residue)
      (:or (> ?gr 100.0) (< ?gr 5.0))))))
(:wat::core::defn :t::test7-or-high [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 150.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?gr :grace-residue)
      (:or (> ?gr 100.0) (< ?gr 5.0))))))
(:wat::core::defn :t::test7-or-mid [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 50.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?gr :grace-residue)
      (:or (> ?gr 100.0) (< ?gr 5.0))))))

; not_inverts
(:wat::core::defn :t::test8-not-grace [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 5.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome) (:not (= ?o "Loss"))))))
(:wat::core::defn :t::test8-not-loss [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Loss" :grace-residue 5.0)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome) (:not (= ?o "Loss"))))))

; where_uses_arbitrary_wat_expression
(:wat::core::defn :t::test9-where-pass [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Graceful" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome)
      (:where (:wat::string::contains? ?o "Grace"))))))

; where_can_fail
(:wat::core::defn :t::test10-where-fail [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Loss" :grace-residue 7.5)]
    (:wat::form::matches? p (:test::PaperResolved
      (= ?o :outcome)
      (:where (:wat::string::contains? ?o "Grace"))))))

; struct_type_mismatch_returns_false
(:wat::core::defn :t::test11-struct-mismatch [] -> :wat::core::bool
  (:wat::core::let [o (:test::Other :x 42)]
    (:wat::form::matches? o
      (:test::PaperResolved
        (= ?gr :grace-residue)
        (> ?gr 5.0)))))

; option_none_subject_returns_false
(:wat::core::defn :t::test12-option-none [] -> :wat::core::bool
  (:wat::core::let [maybe :wat::core::None]
    (:wat::form::matches? maybe
      (:test::PaperResolved
        (= ?gr :grace-residue)
        (> ?gr 5.0)))))

; option_some_subject_unwraps_one_level
(:wat::core::defn :t::test13-option-some [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)
                   maybe (:wat::core::Some p)]
    (:wat::form::matches? maybe
      (:test::PaperResolved
        (= ?gr :grace-residue)
        (> ?gr 5.0)))))

; non_struct_subject_returns_false
(:wat::core::defn :t::test14-non-struct [] -> :wat::core::bool
  (:wat::form::matches? 42
    (:test::PaperResolved
      (= ?gr :grace-residue)
      (> ?gr 5.0))))

; binding_visible_in_later_clauses_including_where
(:wat::core::defn :t::test15-binding-where [] -> :wat::core::bool
  (:wat::core::let [p (:test::PaperResolved :outcome "Grace" :grace-residue 12.5)]
    (:wat::form::matches? p
      (:test::PaperResolved
        (= ?o :outcome)
        (= ?gr :grace-residue)
        (= ?o "Grace")
        (:where (:wat::core::> ?gr 10.0))))))
