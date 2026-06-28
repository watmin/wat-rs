;; tests/reflection/probe_arc255_reflection_parity_user_form.wat
;; Fixture for user_form_carries_guaranteed_baseline (ignored — arc-255 not yet built).
;; A bare user defn (no explicit metadata); metadata-of must return Some(baseline).
(:wat::core::defn :my::f [x <- :wat::core::i64] -> :wat::core::i64 x)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::match (:wat::runtime::metadata-of :my::f) -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))
