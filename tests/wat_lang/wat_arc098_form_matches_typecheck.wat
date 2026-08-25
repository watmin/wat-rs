;; tests/wat_lang/wat_arc098_form_matches_typecheck.wat — co-located fixture.
;; Arc 098 slice 1 — :wat::form::matches? type-check side.
;; Valid patterns; startup must SUCCEED for all tests in this file.

(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])

;; valid_simple_binding_and_comparison
(:wat::core::defn :t::test1-binding-cmp [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?outcome :outcome)
      (= ?grace-residue :grace-residue)
      (= ?outcome "Grace")
      (> ?grace-residue 5.0))))

;; valid_logical_combinators
(:wat::core::defn :t::test2-logical [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?outcome :outcome)
      (= ?grace-residue :grace-residue)
      (:and
        (= ?outcome "Grace")
        (:or
          (> ?grace-residue 5.0)
          (< ?grace-residue 0.0))
        (:not (= ?outcome "Loss"))))))

;; valid_where_escape_returns_bool
(:wat::core::defn :t::test3-where [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?outcome :outcome)
      (:where (:wat::string::contains? ?outcome "Grace")))))
