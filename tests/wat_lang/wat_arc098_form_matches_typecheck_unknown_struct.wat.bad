;; Negative fixture: unknown struct type in pattern → check error.
;; Used by test: rejects_unknown_struct_type

(:wat::core::defstruct :test::PaperResolved
  [outcome <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :t::probe [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (:test::DoesNotExist
      (= ?o :outcome))))
