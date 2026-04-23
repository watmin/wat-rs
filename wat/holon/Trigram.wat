;; :wat::holon::Trigram — triples, per 058-013 (Ngram 3 xs shortcut).

(:wat::core::defmacro
  (:wat::holon::Trigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>>)
  `(:wat::holon::Ngram 3 ,xs))
