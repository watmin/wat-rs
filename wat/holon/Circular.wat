;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Circular)
;;
;; :wat::holon::Circular — stdlib macro for cyclic quantities.
;;
;; (Circular value period) encodes a cyclic quantity as a point on the
;; unit circle by Blending two reserved basis atoms (cos-basis and
;; sin-basis) with weights cos(θ) and sin(θ), where θ = 2π · value / period.
;; Hour 23 and hour 0 are adjacent on the circle; Blend's Option B
;; (independent real-valued weights) is exactly what this needs —
;; cos(π/4) + sin(π/4) ≈ 1.414, not 1.
;;
;; Arith is binary: `(* 2 pi (/ v p))` becomes nested binary
;; `(:wat::core::*` + `:wat::core::/)` calls. Polymorphic forms used;
;; the typed-strict `:wat::f64::*` and `:wat::f64::/`
;; remain available when callers want the type-guard behavior.
;; `:wat::math::pi` is a nullary primitive called as
;; `(:wat::math::pi)`; let bindings carry explicit
;; `:wat::core::f64` types.

(:wat::core::defmacro :wat::holon::Circular
  [value  <- :wat::WatAST
   period <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::let
     [frac
       (:wat::core::/ ~value ~period)
      two-pi
       (:wat::core::* 2.0 (:wat::math::pi))
      theta
       (:wat::core::* two-pi frac)]
     (:wat::holon::Blend
       (:wat::holon::to-holon :wat::std::circular-cos-basis)
       (:wat::holon::to-holon :wat::std::circular-sin-basis)
       (:wat::math::cos theta)
       (:wat::math::sin theta))))
