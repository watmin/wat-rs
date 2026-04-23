;; :wat::std::Sequential — bind-chain with positional Permute per
;; 058-009's reframe.
;;
;; (Sequential [a])       = a
;; (Sequential [a b])     = Bind(a, Permute(b, 1))
;; (Sequential [a b c])   = Bind(Bind(a, Permute(b, 1)), Permute(c, 2))
;; (Sequential [a b c d]) = Bind(Bind(Bind(a, Permute(b, 1)), Permute(c, 2)), Permute(d, 3))
;;
;; Position is carried by Permute at each non-zero index; item 0
;; stays un-permuted. The nested Bind composition creates a compound
;; (strict identity; exact sequence match). Two sequences with the
;; same items in different order produce different compound vectors.
;;
;; Expansion strategy (deviation from proposal's conceptual sketch):
;; use `map-with-index` to attach positions, then `foldl` to bind-chain
;; over tail from head. Uses existing core + std::list combinators
;; (no new primitives).

(:wat::core::defmacro
  (:wat::std::Sequential
    (items :AST<List<wat::holon::HolonAST>>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::core::let*
     (((positioned :Vec<wat::holon::HolonAST>)
       (:wat::std::list::map-with-index ,items
         (:wat::core::lambda ((item :wat::holon::HolonAST) (i :i64) -> :wat::holon::HolonAST)
           (:wat::core::if (:wat::core::= i 0) -> :wat::holon::HolonAST
             item
             (:wat::holon::Permute item i))))))
     (:wat::core::foldl
       (:wat::core::rest positioned)
       (:wat::core::first positioned)
       (:wat::core::lambda ((acc :wat::holon::HolonAST) (x :wat::holon::HolonAST) -> :wat::holon::HolonAST)
         (:wat::holon::Bind acc x)))))
