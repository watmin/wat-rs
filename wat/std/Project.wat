;; :wat::std::Project — Gram-Schmidt project step per 058-005.
;;
;; (Project x y) = x's component along y's direction
;;               = x - Reject(x, y)
;; Equivalently: ((x·y)/(y·y)) · y — the shadow x casts on y's axis.
;;
;; Invariant: (Project x y) + (Reject x y) = x. The Gram-Schmidt duo.
;;
;; Production-cited: engram matching — project(packet, baseline_components)
;; reconstructs the observation as the subspace sees it.

(:wat::core::defmacro
  (:wat::std::Project
    (x :AST<holon::HolonAST>)
    (y :AST<holon::HolonAST>)
    -> :AST<holon::HolonAST>)
  `(:wat::std::Subtract ,x (:wat::std::Reject ,x ,y)))
