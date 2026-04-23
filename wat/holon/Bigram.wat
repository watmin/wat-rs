;; :wat::holon::Bigram — pairs, per 058-013 (Ngram 2 xs shortcut).

(:wat::core::defmacro
  (:wat::holon::Bigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>>)
  `(:wat::holon::Ngram 2 ,xs))
