;; Fixture: test 8 — recursive splice: top-level let → do → def.
(:wat::core::let
  [x 1]
  (:wat::core::do
    (:wat::core::def :a x)
    (:wat::core::def :b (:wat::core::i64::* x 2))))
