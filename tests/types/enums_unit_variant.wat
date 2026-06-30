;; enums_unit_variant.wat — unit variant construction + match.
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Green :Blue)
(:wat::core::defn :my::pick [] -> :my::Color :my::Color::Green)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:my::pick) -> :wat::core::nil
    (:my::Color::Red   (:wat::kernel::println "red"))
    (:my::Color::Green (:wat::kernel::println "green"))
    (:my::Color::Blue  (:wat::kernel::println "blue"))))
