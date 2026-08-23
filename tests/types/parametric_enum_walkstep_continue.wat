;; parametric_enum_walkstep_continue.wat — WalkStep::Continue parametric inference.
(:wat::core::defn :my::test::wrap [n <- :wat::core::i64] -> (:wat::eval::WalkStep :- [:wat::core::i64]) (:wat::eval::WalkStep::Continue n))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [wrapped (:my::test::wrap 7)]
    7))
