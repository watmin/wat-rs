;; parametric_enum_walkstep_skip.wat — WalkStep::Skip parametric inference.
(:wat::core::defn :my::test::halt [n <- :wat::core::i64] -> (:wat::eval::WalkStep :- [:wat::core::i64])
  (:wat::eval::WalkStep::Skip
    (:wat::holon::leaf 999)
    n))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [halted (:my::test::halt 3)]
    3))
