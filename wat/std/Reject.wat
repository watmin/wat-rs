;; :wat::std::Reject — Gram-Schmidt reject step per 058-005.
;;
;; (Reject x y) = x - ((x·y)/(y·y)) · y
;; The component of x orthogonal to y.
;;
;; Expands to Blend with the second weight computed at runtime from
;; the dot-product ratio. The negation is spelled as binary
;; (:wat::core::f64::- 0.0 ratio) since wat arith is typed and binary
;; post the 2026-04-19 split.
;;
;; Production-cited: DDoS sidecar's core detection mechanism
;; (Challenge 010, F1=1.000) — reject(packet, baseline_subspace).
;; Engram matching — residual vs subspace.

(:wat::core::defmacro
  (:wat::std::Reject
    (x :AST<wat::holon::HolonAST>)
    (y :AST<wat::holon::HolonAST>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Blend
     ,x
     ,y
     1.0
     (:wat::core::f64::- 0.0
       (:wat::core::f64::/ (:wat::holon::dot ,x ,y)
                           (:wat::holon::dot ,y ,y)))))
