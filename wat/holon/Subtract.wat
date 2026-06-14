;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Subtract)
;;
;; :wat::holon::Subtract — linear component removal.
;;
;; (Subtract x y) expands to (Blend x y 1 -1): anchor x, invert y.
;; The canonical `Blend(_, _, 1, -1)` idiom. An earlier `Difference`
;; name was REJECTED — one name per operation; Subtract wins.

(:wat::core::defmacro :wat::holon::Subtract
  [x <- :wat::WatAST
   y <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Blend ~x ~y 1.0 -1.0))
