;; ord_enum_bad.wat — user enum not in orderable class. Must FAIL.
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Green :Blue)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< :my::Color::Red :my::Color::Blue))
