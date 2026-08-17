;; wat-tests/interpolate.wat — arc 284 :wat::core::string::interpolate deftest.
;;
;; Runtime interpolation: named slots, unquoted render (String/i64), {{ }} escape.
;; Strict-error cases require startup_from_source failure testing (probe_arc284_interpolate.rs).
;; The expand-time property is proven by the probe (interpolate_is_legal_at_expand_time).

(:wat::test::deftest :wat-tests::interpolate::runtime-named-unquoted-escaped

  (:wat::test::assert-eq
    (:wat::core::string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5)
    "x::5 {lit}"))

;; Stone 279.4 — `interpolate` renders ANYTHING, not just the five-arm scalar
;; domain `render_unquoted` used to accept. `check.rs:13930-13935` already
;; promised value slots accept any str-renderable type "because the intrinsic
;; renders them unquoted at runtime" — before this stone that was false for a
;; record, which type-checked and then raised `TypeMismatch`. Renders through
;; the same door `str`/`join` use (`render_str_total`), so a record renders by
;; NAME (`{:x 1}`), never positionally (`{:field-0 1}`).
(:wat::core::defrecord :wat-tests::interpolate::Rec [x <- :wat::core::i64])

(:wat::test::deftest :wat-tests::interpolate::runtime-record-named-fields

  (:wat::test::assert-eq
    (:wat::core::string::interpolate "{r}" :r (:wat-tests::interpolate::Rec :x 1))
    "#wat-tests.interpolate/Rec {:x 1}"))
