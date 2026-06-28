;; 293.4b NEGATIVE — a record that does NOT satisfy :t::Shape must be rejected at check time.
;;
;; :t::NoArea has no `defn :t::NoArea/area` — it does not satisfy :t::Shape.
;; Passing it to (:t::Shape/area ...) must fail at check time with a TypeMismatch
;; (receiver type :t::NoArea is not assignable to the surface :t::Shape).

(:wat::core::defsurface :t::Shape
  [(area [self] -> :wat::core::f64)])

(:wat::core::defrecord :t::NoArea [x <- :wat::core::f64])

;; This should be rejected at check time: :t::NoArea does not satisfy :t::Shape
;; (no `defn :t::NoArea/area` exists), so the receiver type mismatch fires.
(:wat::core::defn :t::bad-surface-call [] -> :wat::core::f64
  (:t::Shape/area (:t::NoArea 1.0)))
