;; :wat::holon::Ngram — n-wise adjacency per 058-013.
;;
;; (Ngram n xs) = (Bundle (map (window xs n) Sequential))
;;
;; Slides a size-n window across xs, encodes each window with
;; Sequential (bind-chain), bundles every window's compound into one
;; composite holon.
;;
;; The macro expansion references :wat::holon::Sequential at parse
;; time — the macro-expander recursively expands it inside the
;; lambda, so the emitted AST carries Sequential's let*-foldl
;; directly with no runtime call hop.
;;
;; Edge cases per Q2: n <= 0 produces an empty bundle (zero vector);
;; n > xs.len() produces an empty bundle (no window fits).

;; Returns the Bundle's raw Result — caller handles capacity. Per
;; the 2026-04-19 Bundle-Result slice: every stdlib form that expands
;; to Bundle inherits Bundle's Result wrap. Callers either match
;; explicitly or propagate with `:wat::core::try`.

(:wat::core::defmacro
  (:wat::holon::Ngram
    (n :AST<i64>)
    (xs :AST<List<wat::holon::HolonAST>>)
    -> :AST<Result<wat::holon::HolonAST,wat::holon::CapacityExceeded>>)
  `(:wat::holon::Bundle
     (:wat::core::map
       (:wat::std::list::window ,xs ,n)
       (:wat::core::lambda ((window :Vec<wat::holon::HolonAST>) -> :wat::holon::HolonAST)
         (:wat::holon::Sequential window)))))
