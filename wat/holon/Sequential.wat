;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Sequential)
;;
;; :wat::holon::Sequential — bind-chain with positional Permute.
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
;; Expansion strategy: use `map-with-index` to attach positions, then
;; `foldl` to bind-chain over tail from head. Uses existing core +
;; std::list combinators (no new primitives).

(:wat::core::defmacro :wat::holon::Sequential
  [items <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::let
     [positioned
       (:wat::std::list::map-with-index ~items
         (:wat::core::fn [item <- :wat::holon::HolonAST i <- :wat::core::i64] -> :wat::holon::HolonAST
           (:wat::core::if (:wat::core::= i 0) 
             item
             (:wat::holon::Permute item i))))]
     ;; use get for the Option-returning safe path; arc-278 flipped first to bare-raising.
     ;; Sequential expects non-empty input by contract; the :None arm is defensive.
     (:wat::core::match (:wat::core::get positioned 0) 
       ((:wat::core::Some head)
         (:wat::core::foldl
           (:wat::core::fn [acc <- :wat::holon::HolonAST x <- :wat::holon::HolonAST] -> :wat::holon::HolonAST
             (:wat::holon::Bind acc x))
           head
           (:wat::core::rest positioned)))
       (:wat::core::None (:wat::holon::to-holon "Sequential-empty-input")))))
