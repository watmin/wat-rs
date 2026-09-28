;; parametric_enum_walk_visitor.wat — full walker pattern, frozen + type-checked.
(:wat::core::defn :my::test::count-visit [acc <- wat.type/i64 form <- wat.type/AST step <- :wat::eval::StepResult] -> (:wat::eval::WalkStep :- [wat.type/i64]) (:wat::eval::WalkStep.Continue {:acc (:wat::i64::+ acc 1)}))
(:wat::core::defn :my::compute [] -> wat.type/i64
  (:wat::core::match
    (:wat::eval::walk
      (:wat::core::quote
        (:wat::holon::Bind
          (:wat::holon::to-holon "k")
          (:wat::holon::to-holon "v")))
      0
      :my::test::count-visit) 
    [:wat::core::Result.Ok {:value pair}
      (:wat::core::second pair)]
    [:wat::core::Result.Err {:error _e} -1]))
