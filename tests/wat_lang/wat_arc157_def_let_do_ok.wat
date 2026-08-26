;; Fixture: test 8 — recursive splice: top-level let → do → def.
(:wat::core::let
  [x 1]
  (:wat::core::do
    (:wat::core::def :wat-arc157-def-let-do-ok::a x)
    (:wat::core::def :wat-arc157-def-let-do-ok::b (:wat::i64::* x 2))))
