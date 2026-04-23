;; :wat::holon::Trigram — triples, per 058-013 (Ngram 3 xs shortcut).

(:wat::core::defmacro
  (:wat::holon::Trigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<wat::holon::BundleResult>)
  `(:wat::holon::Ngram 3 ,xs))
