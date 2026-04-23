;; :wat::std::Trigram — triples, per 058-013 (Ngram 3 xs shortcut).

(:wat::core::defmacro
  (:wat::std::Trigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>>)
  `(:wat::std::Ngram 3 ,xs))
