;; :wat::std::Ngram — n-wise adjacency per 058-013.
;;
;; (Ngram n xs) = (Bundle (map (window xs n) Sequential))
;;
;; Slides a size-n window across xs, encodes each window with
;; Sequential (bind-chain), bundles every window's compound into one
;; composite holon.
;;
;; The macro expansion references :wat::std::Sequential at parse
;; time — the macro-expander recursively expands it inside the
;; lambda, so the emitted AST carries Sequential's let*-foldl
;; directly with no runtime call hop.
;;
;; Edge cases per Q2: n <= 0 produces an empty bundle (zero vector);
;; n > xs.len() produces an empty bundle (no window fits).

(:wat::core::defmacro
  (:wat::std::Ngram
    (n :AST<i64>)
    (xs :AST<List<holon::HolonAST>>)
    -> :AST<holon::HolonAST>)
  `(:wat::algebra::Bundle
     (:wat::core::map
       (:wat::std::list::window ,xs ,n)
       (:wat::core::lambda ((window :Vec<holon::HolonAST>) -> :holon::HolonAST)
         (:wat::std::Sequential window)))))
