;; wat-tests/interpolate.wat — arc 284 :wat::core::string::interpolate deftest.
;;
;; Runtime interpolation: named slots, unquoted render (String/i64), {{ }} escape.
;; Strict-error cases require startup_from_source failure testing (probe_arc284_interpolate.rs).
;; The expand-time property is proven by the probe (interpolate_is_legal_at_expand_time).

(:wat::test::deftest :wat-tests::interpolate::runtime-named-unquoted-escaped
  
  (:wat::test::assert-eq
    (:wat::core::string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5)
    "x::5 {lit}"))
