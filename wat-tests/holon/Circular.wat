;; wat-tests/holon/Circular.wat — tests for wat/holon/Circular.wat.
;;
;; :wat::holon::Circular (058-018) expands to
;;   (Blend (Atom :cos-basis) (Atom :sin-basis) (cos theta) (sin theta))
;; where theta = 2π·(v/period). Adjacent values on the cycle produce
;; near-neighbor vectors (presence above floor); antipodal values
;; produce far vectors (presence below floor). The load-bearing
;; property for encoding time-of-day, hour-of-week, etc.

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::deftest :wat-tests::holon::Circular::test-adjacent-hours-are-near 1024 :error
  ()
  (:wat::core::let*
    (((h0  :wat::holon::HolonAST) (:wat::holon::Circular  0.0 24.0))
     ((h23 :wat::holon::HolonAST) (:wat::holon::Circular 23.0 24.0)))
    (:wat::test::assert-eq (:wat::holon::presence? h0 h23) true)))

(:wat::test::deftest :wat-tests::holon::Circular::test-antipodal-hours-are-far 1024 :error
  ()
  (:wat::core::let*
    (((h0  :wat::holon::HolonAST) (:wat::holon::Circular  0.0 24.0))
     ((h12 :wat::holon::HolonAST) (:wat::holon::Circular 12.0 24.0)))
    (:wat::test::assert-eq (:wat::holon::presence? h0 h12) false)))
