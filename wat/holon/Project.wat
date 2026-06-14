;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Reject)
;;
;; :wat::holon::Project — Gram-Schmidt project step.
;;
;; (Project x y) = x's component along y's direction
;;               = x - Reject(x, y)
;; Equivalently: ((x·y)/(y·y)) · y — the shadow x casts on y's axis.
;;
;; Invariant: (Project x y) + (Reject x y) = x. The Gram-Schmidt duo.
;;
;; Production-cited: engram matching — project(packet, baseline_components)
;; reconstructs the observation as the subspace sees it.

(:wat::core::defmacro :wat::holon::Project
  [x <- :wat::WatAST
   y <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Subtract ~x (:wat::holon::Reject ~x ~y)))
