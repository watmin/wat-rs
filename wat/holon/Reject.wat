;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Reject)
;;
;; :wat::holon::Reject — Gram-Schmidt reject step.
;;
;; (Reject x y) = x - ((x·y)/(y·y)) · y
;; The component of x orthogonal to y.
;;
;; Expands to Blend with the second weight computed at runtime from
;; the dot-product ratio. The negation is spelled as binary
;; (:wat::core::- 0.0 ratio) since wat arith is binary — there is
;; no unary negate. Polymorphic form used; the typed-strict
;; :wat::f64::- remains available for callers who want the
;; type-guard behavior.
;;
;; Production-cited: DDoS sidecar's core detection mechanism
;; (Challenge 010, F1=1.000) — reject(packet, baseline_subspace).
;; Engram matching — residual vs subspace.
;;
;; Arc 278 the cosine outcome wall — `dot` now returns
;; DotOutcome (Computed[product]/DimensionMismatch[expected,got]),
;; not a bare f64, so both dot calls the expansion generates must be faced.
;; `dot y y` compares y's dimension to itself — DimensionMismatch there is
;; PROVABLY unreachable (a value's own dimension always equals itself), but
;; the match still names it (no `_` wildcard, doctrine-illegal on an enum
;; scrutinee). `dot x y` CAN mismatch if a caller ever passes x/y of
;; differing dimension — that is a genuine call-site bug in a formula macro
;; whose return type (a Blend HolonAST node with an f64 weight slot) has no
;; honest way to carry the fact forward, so both mismatch arms keep the
;; ORIGINAL behavior (a raise) via the established `Result/expect`-on-a-
;; freshly-built-`Err` idiom (see `wat-tests/core/result-expect.wat`), rather
;; than fabricating a weight.
(:wat::core::defmacro :wat::holon::Reject
  [x <- :wat::WatAST
   y <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Blend
     ~x
     ~y
     1.0
     (:wat::core::- 0.0
       (:wat::core::let
         [dot-xy (:wat::holon::dot ~x ~y)
          dot-yy (:wat::holon::dot ~y ~y)]
         (:wat::core::match dot-xy
           ((:wat::holon::DotOutcome::Computed nxy)
             (:wat::core::match dot-yy
               ((:wat::holon::DotOutcome::Computed nyy)
                 (:wat::core::/ nxy nyy))
               ((:wat::holon::DotOutcome::DimensionMismatch _e _g)
                 (:wat::core::Result/expect
                   (:wat::core::Err "Reject: dot(y, y) dimension mismatch — unreachable, a value's dimension always equals itself")
                   "Reject: dot(y, y) dimension mismatch"))))
           ((:wat::holon::DotOutcome::DimensionMismatch _e _g)
             (:wat::core::Result/expect
               (:wat::core::Err "Reject: dot(x, y) dimension mismatch — x and y must share the same dimension")
               "Reject: dot(x, y) dimension mismatch")))))))
