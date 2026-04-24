;; wat-tests/holon/ReciprocalLog.wat — arc 034 tests.
;;
;; Tests :wat::holon::ReciprocalLog — stdlib macro over Log with
;; reciprocal bounds. Four outstanding tests anchoring the
;; macro's specific claims:
;;
;; 1. Expansion equivalence — (ReciprocalLog n v) coincides with
;;    (Log v (/ 1.0 n) n). Proves the macro is pure sugar.
;; 2. Reference (value = 1.0) self-coincidence.
;; 3. Log-symmetric distinguishability — value=1/n and value=n
;;    encode non-coincident with the v=1.0 reference (saturation
;;    at both ends).
;; 4. Different N distinguishes for the same value — (ReciprocalLog 2 1.5)
;;    and (ReciprocalLog 10 1.5) encode different positions along
;;    their respective gradients, so they don't coincide with the
;;    reference identically.

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :deftest ())

;; ─── 1. expansion equivalence ──────────────────────────────────

(:deftest :wat-tests::holon::ReciprocalLog::test-expansion-matches-explicit-log
  (:wat::core::let*
    (((sugar :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 1.5))
     ((explicit :wat::holon::HolonAST)
      (:wat::holon::Log 1.5 0.5 2.0)))
    (:wat::test::assert-eq
      (:wat::holon::coincident? sugar explicit)
      true)))

;; ─── 2. reference value = 1.0 self-coincidence ─────────────────

(:deftest :wat-tests::holon::ReciprocalLog::test-value-1-coincides-with-itself
  (:wat::core::let*
    (((a :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 1.0))
     ((b :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 1.0)))
    (:wat::test::assert-eq
      (:wat::holon::coincident? a b)
      true)))

;; ─── 3. value = n saturates distinguishably from value = 1 ─────

(:deftest :wat-tests::holon::ReciprocalLog::test-upper-bound-not-coincident-with-reference
  (:wat::core::let*
    (((ref :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 1.0))
     ((upper :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 2.0)))
    (:wat::test::assert-eq
      (:wat::holon::coincident? ref upper)
      false)))

;; ─── 4. different N for same value encodes differently ─────────

(:deftest :wat-tests::holon::ReciprocalLog::test-different-n-differs-for-same-value
  (:wat::core::let*
    (((tight :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 2.0 1.5))
     ((wide :wat::holon::HolonAST)
      (:wat::holon::ReciprocalLog 10.0 1.5)))
    ;; At N=2, value=1.5 is near the upper saturation (75% of the
    ;; gradient). At N=10, value=1.5 is near the center (just
    ;; above value=1.0). Different positions → non-coincident.
    (:wat::test::assert-eq
      (:wat::holon::coincident? tight wide)
      false)))
