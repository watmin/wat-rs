;; enums_wildcard_arm.wat — wildcard arm satisfies exhaustiveness.
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Green :Blue)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match :my::Color::Blue -> :wat::core::nil
    (:my::Color::Red (:wat::kernel::println "red"))
    (_               (:wat::kernel::println "other"))))
