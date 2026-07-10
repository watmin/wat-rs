;; Negative fixture: unknown field in binding clause → check error.
;; Used by test: rejects_unknown_field

(:wat::core::defstruct :test::PaperResolved
  [outcome <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :t::probe [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::PaperResolved
      (= ?o :unknown-field))))
