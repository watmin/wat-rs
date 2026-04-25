;; :wat::holon::Bigram — pairs, per 058-013 (Ngram 2 xs shortcut).
;;
;; n=2 is the most-used adjacency size (pair-wise transitions in
;; sequences, indicator A-then-B rhythms, before-and-after
;; differences). The named form lets readers see "encoding pairs"
;; at the call site without pattern-matching on Ngram's `n`
;; parameter. Pure sugar — same semantics as `(Ngram 2 xs)`.

(:wat::core::defmacro
  (:wat::holon::Bigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<wat::holon::BundleResult>)
  `(:wat::holon::Ngram 2 ,xs))
