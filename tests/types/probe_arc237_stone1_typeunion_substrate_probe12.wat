;; Fixture probe 12: typeunion-typed arg accepts member-typed value (bounded existential unify).
(:wat::core::typeunion :my::IorF [:wat::core::i64 :wat::core::f64])
(:wat::core::defn :my::identity [x <- :my::IorF] -> :my::IorF x)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:my::identity 42)
    (:my::identity 3.14)
    nil))
