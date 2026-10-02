;; Fixture probe 12: typeunion-typed arg accepts member-typed value (bounded existential unify).
(:wat::core::typeunion :my::IorF [wat.type/i64 wat.type/f64])
(:wat::core::defn :my::identity [x <- :my::IorF] -> :my::IorF x)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::do
    (:my::identity 42)
    (:my::identity 3.14)
    nil))
