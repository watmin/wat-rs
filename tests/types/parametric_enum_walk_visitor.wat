;; parametric_enum_walk_visitor.wat — full walker pattern, frozen + type-checked.
(:wat::core::defn :my::test::count-visit [acc <- :wat::core::i64 form <- :wat::WatAST step <- :wat::eval::StepResult] -> (:wat::eval::WalkStep :- [:wat::core::i64]) (:wat::eval::WalkStep::Continue (:wat::core::i64::+ acc 1)))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::match
    (:wat::eval::walk
      (:wat::core::quote
        (:wat::holon::Bind
          (:wat::holon::to-holon "k")
          (:wat::holon::to-holon "v")))
      0
      :my::test::count-visit) 
    ((:wat::core::Ok pair)
      (:wat::core::second pair))
    ((:wat::core::Err _e) -1)))
