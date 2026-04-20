;; :wat::std::Trigram — triples, per 058-013 (Ngram 3 xs shortcut).

(:wat::core::defmacro
  (:wat::std::Trigram
    (xs :AST<List<holon::HolonAST>>)
    -> :AST<Result<holon::HolonAST,wat::algebra::CapacityExceeded>>)
  `(:wat::std::Ngram 3 ,xs))
