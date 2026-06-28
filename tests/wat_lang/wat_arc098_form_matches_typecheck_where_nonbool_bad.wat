;; Negative fixture: where-body is non-bool (String) → check error.
;; Used by test: rejects_where_body_non_bool

(:wat::core::defstruct :test::PaperResolved
  [outcome <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :t::probe [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?o :outcome)
      (:where ?o))))
