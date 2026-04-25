;; :wat::holon::Trigram — triples, per 058-013 (Ngram 3 xs shortcut).
;;
;; The canonical bind-chain rhythm shape — three-element windows
;; capture an A→B→C transition pattern that pairs miss (Bigrams
;; can't distinguish "rise then fall" from "fall then rise" without
;; an extra name). The trading lab's `indicator-rhythm` builder
;; reaches for Trigrams over candle windows specifically for this
;; reason. Pure sugar — same semantics as `(Ngram 3 xs)`.

(:wat::core::defmacro
  (:wat::holon::Trigram
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<wat::holon::BundleResult>)
  `(:wat::holon::Ngram 3 ,xs))
