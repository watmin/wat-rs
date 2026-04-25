;; :wat::holon::Sequential — bind-chain with positional Permute per
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
  (:wat::holon::Sequential
    (items :AST<List<wat::holon::HolonAST>>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::core::let*
     (((positioned :wat::holon::Holons)
       (:wat::std::list::map-with-index ,items
         (:wat::core::lambda ((item :wat::holon::HolonAST) (i :i64) -> :wat::holon::HolonAST)
           (:wat::core::if (:wat::core::= i 0) -> :wat::holon::HolonAST
             item
             (:wat::holon::Permute item i))))))
     ;; first returns Option<HolonAST> via arc 047. Sequential
     ;; expects non-empty input by contract; the :None arm is
     ;; unreachable but the type checker demands totality.
     (:wat::core::match (:wat::core::first positioned) -> :wat::holon::HolonAST
       ((Some head)
         (:wat::core::foldl
           (:wat::core::rest positioned)
           head
           (:wat::core::lambda ((acc :wat::holon::HolonAST) (x :wat::holon::HolonAST) -> :wat::holon::HolonAST)
             (:wat::holon::Bind acc x))))
       (:None (:wat::holon::Atom "Sequential-empty-input")))))
