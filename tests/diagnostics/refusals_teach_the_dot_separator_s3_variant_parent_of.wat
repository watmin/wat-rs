(:wat::core::defenum :u::E :wat::enum::Pure :A [x <- :wat::core::i64] :B)
(:wat::core::defn :u::f [] -> (:wat::core::Option :- [:wat::core::keyword])
  (:wat::runtime::variant-parent-of :u::E.A))
