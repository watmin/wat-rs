;; :wat::holon::Circular — stdlib macro per 058-018.
;;
;; (Circular value period) encodes a cyclic quantity as a point on the
;; unit circle by Blending two reserved basis atoms (cos-basis and
;; sin-basis) with weights cos(θ) and sin(θ), where θ = 2π · value / period.
;; Hour 23 and hour 0 are adjacent on the circle; Blend's Option B
;; (independent real-valued weights) is exactly what this needs —
;; cos(π/4) + sin(π/4) ≈ 1.414, not 1.
;;
;; Deviations from the proposal's body shape:
;;   - arith is binary: the proposal's `(* 2 pi (/ v p))` trinary form
;;     becomes nested binary `(:wat::core::*` + `:wat::core::/`)
;;     calls. Polymorphic forms used (arc 050); the typed strict
;;     `:wat::core::f64::*,2` and `:wat::core::f64::/,2` remain available
;;     when callers want the type-guard behavior.
;;   - `:wat::std::math::pi` was written bare in the proposal; it's a
;;     nullary primitive, called as `(:wat::std::math::pi)` here.
;;   - let* bindings carry explicit `:wat::core::f64` types.
;; Same math, enforcement-correct wat.

(:wat::core::defmacro
  (:wat::holon::Circular
    (value :AST<wat::core::f64>)
    (period :AST<wat::core::f64>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::core::let*
     (((frac :wat::core::f64)
       (:wat::core::/ ,value ,period))
      ((two-pi :wat::core::f64)
       (:wat::core::* 2.0 (:wat::std::math::pi)))
      ((theta :wat::core::f64)
       (:wat::core::* two-pi frac)))
     (:wat::holon::Blend
       (:wat::holon::Atom :wat::std::circular-cos-basis)
       (:wat::holon::Atom :wat::std::circular-sin-basis)
       (:wat::std::math::cos theta)
       (:wat::std::math::sin theta))))
