;; Negative fixture: unknown constraint head :foo → check error.
;; Used by test: rejects_unknown_constraint_head

(:wat::core::defstruct :test::PaperResolved
  [outcome <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :t::probe [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?o :outcome)
      (:foo ?o "x"))))
