;; enums_missing_variant_bad.wat — missing variant arm (Blue) must fail non-exhaustive. Must FAIL.
(:wat::core::defenum :my::Color :Red :Green :Blue)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match :my::Color::Red -> :wat::core::i64
    (:my::Color::Red   1)
    (:my::Color::Green 2)))
