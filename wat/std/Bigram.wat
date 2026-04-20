;; :wat::std::Bigram — pairs, per 058-013 (Ngram 2 xs shortcut).

(:wat::core::defmacro
  (:wat::std::Bigram
    (xs :AST<List<holon::HolonAST>>)
    -> :AST<Result<holon::HolonAST,wat::algebra::CapacityExceeded>>)
  `(:wat::std::Ngram 2 ,xs))
