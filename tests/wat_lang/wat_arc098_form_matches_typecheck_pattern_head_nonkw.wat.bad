;; Negative fixture: pattern head is a number literal (not a keyword) → check error.
;; Used by test: rejects_pattern_head_non_keyword

(:wat::core::defstruct :test::PaperResolved
  [outcome <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :t::probe [p <- :test::PaperResolved] -> :wat::core::bool
  (:wat::form::matches? p
    (42
      (= ?o :outcome))))
