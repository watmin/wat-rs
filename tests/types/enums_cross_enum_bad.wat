;; enums_cross_enum_bad.wat — cross-enum variant pattern must be rejected. Must FAIL.
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Green)
(:wat::core::defenum :my::Side :wat::enum::Pure  :Buy :Sell)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match :my::Color::Red -> :wat::core::i64
    (:my::Side::Buy  1)
    (:my::Color::Red 2)
    (:my::Color::Green 3)))
