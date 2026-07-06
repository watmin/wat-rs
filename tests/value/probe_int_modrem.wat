;; Co-located fixture for probe_int_modrem.rs — arc 278 numeric-tower increment,
;; `mod`/`rem`/`quot` for i64 (clj-faithful; see BRIEF-STONE-int-modrem.md).
;;
;; The sign table (the whole point — the three differ ONLY by sign):
;;   quot  — truncate toward zero:      7 3=2   -7 3=-2   7 -3=-2   -7 -3=2
;;   rem   — sign of the DIVIDEND:      7 3=1   -7 3=-1   7 -3=1    -7 -3=-1
;;   mod   — sign of the DIVISOR (floored): 7 3=1  -7 3=2  7 -3=-2  -7 -3=-1
;; Plus the i64::MIN / -1 edge: rem/mod = 0 (quot overflows — asserted from
;; the .rs side since it's an Err, not an Ok value assert-eq can carry).

(:wat::test::deftest' :user::modrem_sign_table ()
  (:wat::core::do
    ;; quot — truncate toward zero
    (:wat::test::assert-eq (:wat::core::quot 7 3) 2)
    (:wat::test::assert-eq (:wat::core::quot -7 3) -2)
    (:wat::test::assert-eq (:wat::core::quot 7 -3) -2)
    (:wat::test::assert-eq (:wat::core::quot -7 -3) 2)
    ;; rem — sign of the dividend
    (:wat::test::assert-eq (:wat::core::rem 7 3) 1)
    (:wat::test::assert-eq (:wat::core::rem -7 3) -1)
    (:wat::test::assert-eq (:wat::core::rem 7 -3) 1)
    (:wat::test::assert-eq (:wat::core::rem -7 -3) -1)
    ;; mod — sign of the divisor (floored)
    (:wat::test::assert-eq (:wat::core::mod 7 3) 1)
    (:wat::test::assert-eq (:wat::core::mod -7 3) 2)
    (:wat::test::assert-eq (:wat::core::mod 7 -3) -2)
    (:wat::test::assert-eq (:wat::core::mod -7 -3) -1)
    ;; i64::MIN / -1 edge: rem/mod are mathematically 0 (checked_rem's `None`
    ;; special-cased, clj-faithful — quot's overflow is asserted from the .rs side)
    (:wat::test::assert-eq (:wat::core::rem -9223372036854775808 -1) 0)
    (:wat::test::assert-eq (:wat::core::mod -9223372036854775808 -1) 0)))
