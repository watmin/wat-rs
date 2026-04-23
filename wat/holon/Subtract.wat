;; :wat::holon::Subtract — linear component removal per 058-019.
;;
;; (Subtract x y) expands to (Blend x y 1 -1): anchor x, invert y.
;; The canonical `Blend(_, _, 1, -1)` idiom. Difference (058-004) was
;; REJECTED — one name per operation; Subtract wins.

(:wat::core::defmacro
  (:wat::holon::Subtract
    (x :AST<wat::holon::HolonAST>)
    (y :AST<wat::holon::HolonAST>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Blend ,x ,y 1.0 -1.0))
